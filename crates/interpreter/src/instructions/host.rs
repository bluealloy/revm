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
    context: &mut InstructionContext<'_, H, WIRE>,
) {
    popn_top!([], top, context.interpreter);
    let address = top.into_address();
    let Some(balance) = context.host.balance(address) else {
        context
            .interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };
    let spec_id = context.interpreter.runtime_flag.spec_id();
    gas!(
        context.interpreter,
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
    context: &mut InstructionContext<'_, H, WIRE>,
) {
    check!(context.interpreter, ISTANBUL);
    gas!(context.interpreter, gas::LOW);

    let Some(balance) = context
        .host
        .balance(context.interpreter.input.target_address())
    else {
        context
            .interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };
    push!(context.interpreter, balance.data);
}

pub fn extcodesize<WIRE: InterpreterTypes, H: Host + ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) {
    popn_top!([], top, context.interpreter);
    let address = top.into_address();
    let Some(code) = context.host.load_account_code(address) else {
        context
            .interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };
    let spec_id = context.interpreter.runtime_flag.spec_id();
    if spec_id.is_enabled_in(BERLIN) {
        gas!(context.interpreter, warm_cold_cost(code.is_cold));
    } else if spec_id.is_enabled_in(TANGERINE) {
        gas!(context.interpreter, 700);
    } else {
        gas!(context.interpreter, 20);
    }

    *top = U256::from(code.len());
}

/// EIP-1052: EXTCODEHASH opcode
pub fn extcodehash<WIRE: InterpreterTypes, H: Host + ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) {
    check!(context.interpreter, CONSTANTINOPLE);
    popn_top!([], top, context.interpreter);
    let address = top.into_address();
    let Some(code_hash) = context.host.load_account_code_hash(address) else {
        context
            .interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };
    let spec_id = context.interpreter.runtime_flag.spec_id();
    if spec_id.is_enabled_in(BERLIN) {
        gas!(context.interpreter, warm_cold_cost(code_hash.is_cold));
    } else if spec_id.is_enabled_in(ISTANBUL) {
        gas!(context.interpreter, 700);
    } else {
        gas!(context.interpreter, 400);
    }
    *top = code_hash.into_u256();
}

pub fn extcodecopy<WIRE: InterpreterTypes, H: Host + ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) {
    popn!(
        [address, memory_offset, code_offset, len_u256],
        context.interpreter
    );
    let address = address.into_address();
    let Some(code) = context.host.load_account_code(address) else {
        context
            .interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };

    let len = as_usize_or_fail!(context.interpreter, len_u256);
    gas_or_fail!(
        context.interpreter,
        gas::extcodecopy_cost(
            context.interpreter.runtime_flag.spec_id(),
            len,
            code.is_cold
        )
    );
    if len == 0 {
        return;
    }
    let memory_offset = as_usize_or_fail!(context.interpreter, memory_offset);
    let code_offset = min(as_usize_saturated!(code_offset), code.len());
    resize_memory!(context.interpreter, memory_offset, len);

    // Note: This can't panic because we resized memory to fit.
    context
        .interpreter
        .memory
        .set_data(memory_offset, code_offset, len, &code);
}

pub fn blockhash<WIRE: InterpreterTypes, H: Host + ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) {
    gas!(context.interpreter, gas::BLOCKHASH);
    popn_top!([], number, context.interpreter);

    let requested_number = *number;
    let block_number = context.host.block_number();

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
        let Some(hash) = context.host.block_hash(as_u64_saturated!(requested_number)) else {
            context.interpreter
                .control
                .set_instruction_result(InstructionResult::FatalExternalError);
            return;
        };
        U256::from_be_bytes(hash.0)
    } else {
        U256::ZERO
    }
}

pub fn sload<WIRE: InterpreterTypes, H: Host + ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) {
    popn_top!([], index, context.interpreter);

    let Some(value) = context
        .host
        .sload(context.interpreter.input.target_address(), *index)
    else {
        context
            .interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };

    gas!(
        context.interpreter,
        gas::sload_cost(context.interpreter.runtime_flag.spec_id(), value.is_cold)
    );
    *index = value.data;
}

pub fn sstore<WIRE: InterpreterTypes, H: Host + ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) {
    require_non_staticcall!(context.interpreter);

    popn!([index, value], context.interpreter);

    let Some(state_load) =
        context
            .host
            .sstore(context.interpreter.input.target_address(), index, value)
    else {
        context
            .interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };

    // EIP-1706 Disable SSTORE with gasleft lower than call stipend
    if context
        .interpreter
        .runtime_flag
        .spec_id()
        .is_enabled_in(ISTANBUL)
        && context.interpreter.control.gas().remaining() <= CALL_STIPEND
    {
        context
            .interpreter
            .control
            .set_instruction_result(InstructionResult::ReentrancySentryOOG);
        return;
    }
    gas!(
        context.interpreter,
        gas::sstore_cost(
            context.interpreter.runtime_flag.spec_id(),
            &state_load.data,
            state_load.is_cold
        )
    );

    context
        .interpreter
        .control
        .gas_mut()
        .record_refund(gas::sstore_refund(
            context.interpreter.runtime_flag.spec_id(),
            &state_load.data,
        ));
}

/// EIP-1153: Transient storage opcodes
/// Store value to transient storage
pub fn tstore<WIRE: InterpreterTypes, H: Host + ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) {
    check!(context.interpreter, CANCUN);
    require_non_staticcall!(context.interpreter);
    gas!(context.interpreter, gas::WARM_STORAGE_READ_COST);

    popn!([index, value], context.interpreter);

    context
        .host
        .tstore(context.interpreter.input.target_address(), index, value);
}

/// EIP-1153: Transient storage opcodes
/// Load value from transient storage
pub fn tload<WIRE: InterpreterTypes, H: Host + ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) {
    check!(context.interpreter, CANCUN);
    gas!(context.interpreter, gas::WARM_STORAGE_READ_COST);

    popn_top!([], index, context.interpreter);

    *index = context
        .host
        .tload(context.interpreter.input.target_address(), *index);
}

pub fn log<const N: usize, H: Host + ?Sized>(
    context: &mut InstructionContext<'_, H, impl InterpreterTypes>,
) {
    require_non_staticcall!(context.interpreter);

    popn!([offset, len], context.interpreter);
    let len = as_usize_or_fail!(context.interpreter, len);
    gas_or_fail!(context.interpreter, gas::log_cost(N as u8, len as u64));
    let data = if len == 0 {
        Bytes::new()
    } else {
        let offset = as_usize_or_fail!(context.interpreter, offset);
        resize_memory!(context.interpreter, offset, len);
        Bytes::copy_from_slice(context.interpreter.memory.slice_len(offset, len).as_ref())
    };
    if context.interpreter.stack.len() < N {
        context
            .interpreter
            .control
            .set_instruction_result(InstructionResult::StackUnderflow);
        return;
    }
    let Some(topics) = context.interpreter.stack.popn::<N>() else {
        context
            .interpreter
            .control
            .set_instruction_result(InstructionResult::StackUnderflow);
        return;
    };

    let log = Log {
        address: context.interpreter.input.target_address(),
        data: LogData::new(topics.into_iter().map(B256::from).collect(), data)
            .expect("LogData should have <=4 topics"),
    };

    context.host.log(log);
}

pub fn selfdestruct<WIRE: InterpreterTypes, H: Host + ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) {
    require_non_staticcall!(context.interpreter);
    popn!([target], context.interpreter);
    let target = target.into_address();

    let Some(res) = context
        .host
        .selfdestruct(context.interpreter.input.target_address(), target)
    else {
        context
            .interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };

    // EIP-3529: Reduction in refunds
    if !context
        .interpreter
        .runtime_flag
        .spec_id()
        .is_enabled_in(LONDON)
        && !res.previously_destroyed
    {
        context
            .interpreter
            .control
            .gas_mut()
            .record_refund(gas::SELFDESTRUCT)
    }

    gas!(
        context.interpreter,
        gas::selfdestruct_cost(context.interpreter.runtime_flag.spec_id(), res)
    );

    context
        .interpreter
        .control
        .set_instruction_result(InstructionResult::SelfDestruct);
}
