use crate::{
    gas::{self, warm_cold_cost, CALL_STIPEND},
    instructions::utility::{IntoAddress, IntoU256},
    interpreter_types::{InputsTr, InterpreterTypes, LoopControl, MemoryTr, RuntimeFlag, StackTr},
    Host, InstructionResult,
};
use core::cmp::min;
use primitives::{hardfork::SpecId::*, Bytes, Log, LogData, B256, BLOCK_HASH_HISTORY, U256};

use super::context::InstructionContext;

pub fn balance<WIRE: InterpreterTypes, H: Host + ?Sized>(
    ctx: &mut InstructionContext<'_, H, WIRE>,
) {
    popn_top!([], top, ctx.interpreter);
    let address = top.into_address();
    let Some(balance) = ctx.host.balance(address) else {
        ctx.interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };
    let spec_id = ctx.interpreter.runtime_flag.spec_id();
    gas!(
        ctx.interpreter,
        if spec_id.is_enabled_in(BERLIN) {
            warm_cold_cost(balance.is_cold)
        } else if spec_id.is_enabled_in(ISTANBUL) {
            // EIP-1884: Repricing for trie-size-dependent opcodes
            700
        } else if spec_id.is_enabled_in(TANGERINE) {
            400
        } else {
            20
        }
    );
    *top = balance.data;
}

/// EIP-1884: Repricing for trie-size-dependent opcodes
pub fn selfbalance<WIRE: InterpreterTypes, H: Host + ?Sized>(
    ctx: &mut InstructionContext<'_, H, WIRE>,
) {
    check!(ctx.interpreter, ISTANBUL);
    gas!(ctx.interpreter, gas::LOW);

    let Some(balance) = ctx.host.balance(ctx.interpreter.input.target_address()) else {
        ctx.interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };
    push!(ctx.interpreter, balance.data);
}

pub fn extcodesize<WIRE: InterpreterTypes, H: Host + ?Sized>(
    ctx: &mut InstructionContext<'_, H, WIRE>,
) {
    popn_top!([], top, ctx.interpreter);
    let address = top.into_address();
    let Some(code) = ctx.host.load_account_code(address) else {
        ctx.interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };
    let spec_id = ctx.interpreter.runtime_flag.spec_id();
    if spec_id.is_enabled_in(BERLIN) {
        gas!(ctx.interpreter, warm_cold_cost(code.is_cold));
    } else if spec_id.is_enabled_in(TANGERINE) {
        gas!(ctx.interpreter, 700);
    } else {
        gas!(ctx.interpreter, 20);
    }

    *top = U256::from(code.len());
}

/// EIP-1052: EXTCODEHASH opcode
pub fn extcodehash<WIRE: InterpreterTypes, H: Host + ?Sized>(
    ctx: &mut InstructionContext<'_, H, WIRE>,
) {
    check!(ctx.interpreter, CONSTANTINOPLE);
    popn_top!([], top, ctx.interpreter);
    let address = top.into_address();
    let Some(code_hash) = ctx.host.load_account_code_hash(address) else {
        ctx.interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };
    let spec_id = ctx.interpreter.runtime_flag.spec_id();
    if spec_id.is_enabled_in(BERLIN) {
        gas!(ctx.interpreter, warm_cold_cost(code_hash.is_cold));
    } else if spec_id.is_enabled_in(ISTANBUL) {
        gas!(ctx.interpreter, 700);
    } else {
        gas!(ctx.interpreter, 400);
    }
    *top = code_hash.into_u256();
}

pub fn extcodecopy<WIRE: InterpreterTypes, H: Host + ?Sized>(
    ctx: &mut InstructionContext<'_, H, WIRE>,
) {
    popn!(
        [address, memory_offset, code_offset, len_u256],
        ctx.interpreter
    );
    let address = address.into_address();
    let Some(code) = ctx.host.load_account_code(address) else {
        ctx.interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };

    let len = as_usize_or_fail!(ctx.interpreter, len_u256);
    gas_or_fail!(
        ctx.interpreter,
        gas::extcodecopy_cost(ctx.interpreter.runtime_flag.spec_id(), len, code.is_cold)
    );
    if len == 0 {
        return;
    }
    let memory_offset = as_usize_or_fail!(ctx.interpreter, memory_offset);
    let code_offset = min(as_usize_saturated!(code_offset), code.len());
    resize_memory!(ctx.interpreter, memory_offset, len);

    // Note: This can't panic because we resized memory to fit.
    ctx.interpreter
        .memory
        .set_data(memory_offset, code_offset, len, &code);
}

pub fn blockhash<WIRE: InterpreterTypes, H: Host + ?Sized>(
    ctx: &mut InstructionContext<'_, H, WIRE>,
) {
    gas!(ctx.interpreter, gas::BLOCKHASH);
    popn_top!([], number, ctx.interpreter);

    let requested_number = *number;
    let block_number = ctx.host.block_number();

    let Some(diff) = block_number.checked_sub(requested_number) else {
        *number = U256::ZERO;
        return;
    };

    let diff = as_u64_saturated!(diff);

    // blockhash should push zero if number is same as current block number.
    if diff == 0 {
        *number = U256::ZERO;
        return;
    }

    *number = if diff <= BLOCK_HASH_HISTORY {
        let Some(hash) = ctx.host.block_hash(as_u64_saturated!(requested_number)) else {
            ctx.interpreter
                .control
                .set_instruction_result(InstructionResult::FatalExternalError);
            return;
        };
        U256::from_be_bytes(hash.0)
    } else {
        U256::ZERO
    }
}

pub fn sload<WIRE: InterpreterTypes, H: Host + ?Sized>(ctx: &mut InstructionContext<'_, H, WIRE>) {
    popn_top!([], index, ctx.interpreter);

    let Some(value) = ctx
        .host
        .sload(ctx.interpreter.input.target_address(), *index)
    else {
        ctx.interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };

    gas!(
        ctx.interpreter,
        gas::sload_cost(ctx.interpreter.runtime_flag.spec_id(), value.is_cold)
    );
    *index = value.data;
}

pub fn sstore<WIRE: InterpreterTypes, H: Host + ?Sized>(ctx: &mut InstructionContext<'_, H, WIRE>) {
    require_non_staticcall!(ctx.interpreter);

    popn!([index, value], ctx.interpreter);

    let Some(state_load) = ctx
        .host
        .sstore(ctx.interpreter.input.target_address(), index, value)
    else {
        ctx.interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };

    // EIP-1706 Disable SSTORE with gasleft lower than call stipend
    if ctx
        .interpreter
        .runtime_flag
        .spec_id()
        .is_enabled_in(ISTANBUL)
        && ctx.interpreter.control.gas().remaining() <= CALL_STIPEND
    {
        ctx.interpreter
            .control
            .set_instruction_result(InstructionResult::ReentrancySentryOOG);
        return;
    }
    gas!(
        ctx.interpreter,
        gas::sstore_cost(
            ctx.interpreter.runtime_flag.spec_id(),
            &state_load.data,
            state_load.is_cold
        )
    );

    ctx.interpreter
        .control
        .gas_mut()
        .record_refund(gas::sstore_refund(
            ctx.interpreter.runtime_flag.spec_id(),
            &state_load.data,
        ));
}

/// EIP-1153: Transient storage opcodes
/// Store value to transient storage
pub fn tstore<WIRE: InterpreterTypes, H: Host + ?Sized>(ctx: &mut InstructionContext<'_, H, WIRE>) {
    check!(ctx.interpreter, CANCUN);
    require_non_staticcall!(ctx.interpreter);
    gas!(ctx.interpreter, gas::WARM_STORAGE_READ_COST);

    popn!([index, value], ctx.interpreter);

    ctx.host
        .tstore(ctx.interpreter.input.target_address(), index, value);
}

/// EIP-1153: Transient storage opcodes
/// Load value from transient storage
pub fn tload<WIRE: InterpreterTypes, H: Host + ?Sized>(ctx: &mut InstructionContext<'_, H, WIRE>) {
    check!(ctx.interpreter, CANCUN);
    gas!(ctx.interpreter, gas::WARM_STORAGE_READ_COST);

    popn_top!([], index, ctx.interpreter);

    *index = ctx
        .host
        .tload(ctx.interpreter.input.target_address(), *index);
}

pub fn log<const N: usize, H: Host + ?Sized>(
    ctx: &mut InstructionContext<'_, H, impl InterpreterTypes>,
) {
    require_non_staticcall!(ctx.interpreter);

    popn!([offset, len], ctx.interpreter);
    let len = as_usize_or_fail!(ctx.interpreter, len);
    gas_or_fail!(ctx.interpreter, gas::log_cost(N as u8, len as u64));
    let data = if len == 0 {
        Bytes::new()
    } else {
        let offset = as_usize_or_fail!(ctx.interpreter, offset);
        resize_memory!(ctx.interpreter, offset, len);
        Bytes::copy_from_slice(ctx.interpreter.memory.slice_len(offset, len).as_ref())
    };
    if ctx.interpreter.stack.len() < N {
        ctx.interpreter
            .control
            .set_instruction_result(InstructionResult::StackUnderflow);
        return;
    }
    let Some(topics) = ctx.interpreter.stack.popn::<N>() else {
        ctx.interpreter
            .control
            .set_instruction_result(InstructionResult::StackUnderflow);
        return;
    };

    let log = Log {
        address: ctx.interpreter.input.target_address(),
        data: LogData::new(topics.into_iter().map(B256::from).collect(), data)
            .expect("LogData should have <=4 topics"),
    };

    ctx.host.log(log);
}

pub fn selfdestruct<WIRE: InterpreterTypes, H: Host + ?Sized>(
    ctx: &mut InstructionContext<'_, H, WIRE>,
) {
    require_non_staticcall!(ctx.interpreter);
    popn!([target], ctx.interpreter);
    let target = target.into_address();

    let Some(res) = ctx
        .host
        .selfdestruct(ctx.interpreter.input.target_address(), target)
    else {
        ctx.interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };

    // EIP-3529: Reduction in refunds
    if !ctx.interpreter.runtime_flag.spec_id().is_enabled_in(LONDON) && !res.previously_destroyed {
        ctx.interpreter
            .control
            .gas_mut()
            .record_refund(gas::SELFDESTRUCT)
    }

    gas!(
        ctx.interpreter,
        gas::selfdestruct_cost(ctx.interpreter.runtime_flag.spec_id(), res)
    );

    ctx.interpreter
        .control
        .set_instruction_result(InstructionResult::SelfDestruct);
}
