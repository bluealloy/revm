use crate::{
    gas::{self, warm_cold_cost, warm_cold_cost_with_delegation, CALL_STIPEND},
    instructions::utility::IntoAddress,
    interpreter::Interpreter,
    interpreter_types::{
        InputsTrait, InterpreterTypes, LoopControl, MemoryTrait, RuntimeFlag, StackTrait,
    },
    Host, InstructionResult,
};
use core::cmp::min;
use primitives::{Bytes, Log, LogData, B256, U256};
use specification::hardfork::SpecId::*;

pub fn balance<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    host: &mut H,
) {
    popn_top!([], top, interpreter);
    let address = top.into_address();
    let Some(balance) = host.balance(address) else {
        interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };
    let spec_id = interpreter.runtime_flag.spec_id();
    gas!(
        interpreter,
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
    interpreter: &mut Interpreter<WIRE>,
    host: &mut H,
) {
    check!(interpreter, ISTANBUL);
    gas!(interpreter, gas::LOW);
    let Some(balance) = host.balance(interpreter.input.target_address()) else {
        interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };
    push!(interpreter, balance.data);
}

pub fn extcodesize<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    host: &mut H,
) {
    popn_top!([], top, interpreter);
    let address = top.into_address();
    let Some(code) = host.code(address) else {
        interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };
    let (code, load) = code.into_components();
    let spec_id = interpreter.runtime_flag.spec_id();
    if spec_id.is_enabled_in(BERLIN) {
        gas!(interpreter, warm_cold_cost_with_delegation(load));
    } else if spec_id.is_enabled_in(TANGERINE) {
        gas!(interpreter, 700);
    } else {
        gas!(interpreter, 20);
    }

    *top = U256::from(code.len());
}

/// EIP-1052: EXTCODEHASH opcode
pub fn extcodehash<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    host: &mut H,
) {
    check!(interpreter, CONSTANTINOPLE);
    popn_top!([], top, interpreter);
    let address = top.into_address();
    let Some(code_hash) = host.code_hash(address) else {
        interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };
    let (code_hash, load) = code_hash.into_components();
    let spec_id = interpreter.runtime_flag.spec_id();
    if spec_id.is_enabled_in(BERLIN) {
        gas!(interpreter, warm_cold_cost_with_delegation(load))
    } else if spec_id.is_enabled_in(ISTANBUL) {
        gas!(interpreter, 700);
    } else {
        gas!(interpreter, 400);
    }
    *top = code_hash.into();
}

pub fn extcodecopy<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    host: &mut H,
) {
    popn!([address, memory_offset, code_offset, len_u256], interpreter);
    let address = address.into_address();
    let Some(code) = host.code(address) else {
        interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };

    let len = as_usize_or_fail!(interpreter, len_u256);
    let (code, load) = code.into_components();
    gas_or_fail!(
        interpreter,
        gas::extcodecopy_cost(interpreter.runtime_flag.spec_id(), len, load)
    );
    if len == 0 {
        return;
    }
    let memory_offset = as_usize_or_fail!(interpreter, memory_offset);
    let code_offset = min(as_usize_saturated!(code_offset), code.len());
    resize_memory!(interpreter, memory_offset, len);

    // Note: this can't panic because we resized memory to fit.
    interpreter
        .memory
        .set_data(memory_offset, code_offset, len, &code);
}

pub fn blockhash<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    host: &mut H,
) {
    gas!(interpreter, gas::BLOCKHASH);
    popn_top!([], number, interpreter);

    let number_u64 = as_u64_saturated!(number);
    let Some(hash) = host.block_hash(number_u64) else {
        interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };
    *number = U256::from_be_bytes(hash.0);
}

pub fn sload<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    host: &mut H,
) {
    popn_top!([], index, interpreter);
    let Some(value) = host.sload(interpreter.input.target_address(), *index) else {
        interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };
    gas!(
        interpreter,
        gas::sload_cost(interpreter.runtime_flag.spec_id(), value.is_cold)
    );
    *index = value.data;
}

pub fn sstore<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    host: &mut H,
) {
    require_non_staticcall!(interpreter);

    popn!([index, value], interpreter);
    let Some(state_load) = host.sstore(interpreter.input.target_address(), index, value) else {
        interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };

    // EIP-1706 Disable SSTORE with gasleft lower than call stipend
    if interpreter.runtime_flag.spec_id().is_enabled_in(ISTANBUL)
        && interpreter.control.gas().remaining() <= CALL_STIPEND
    {
        interpreter
            .control
            .set_instruction_result(InstructionResult::ReentrancySentryOOG);
        return;
    }
    gas!(
        interpreter,
        gas::sstore_cost(
            interpreter.runtime_flag.spec_id(),
            &state_load.data,
            state_load.is_cold
        )
    );

    interpreter.control.gas().record_refund(gas::sstore_refund(
        interpreter.runtime_flag.spec_id(),
        &state_load.data,
    ));
}

/// EIP-1153: Transient storage opcodes
/// Store value to transient storage
pub fn tstore<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    host: &mut H,
) {
    check!(interpreter, CANCUN);
    require_non_staticcall!(interpreter);
    gas!(interpreter, gas::WARM_STORAGE_READ_COST);

    popn!([index, value], interpreter);

    host.tstore(interpreter.input.target_address(), index, value);
}

/// EIP-1153: Transient storage opcodes
/// Load value from transient storage
pub fn tload<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    host: &mut H,
) {
    check!(interpreter, CANCUN);
    gas!(interpreter, gas::WARM_STORAGE_READ_COST);

    popn_top!([], index, interpreter);

    *index = host.tload(interpreter.input.target_address(), *index);
}

pub fn log<const N: usize, H: Host + ?Sized>(
    interpreter: &mut Interpreter<impl InterpreterTypes>,
    host: &mut H,
) {
    require_non_staticcall!(interpreter);

    popn!([offset, len], interpreter);
    let len = as_usize_or_fail!(interpreter, len);
    gas_or_fail!(interpreter, gas::log_cost(N as u8, len as u64));
    let data = if len == 0 {
        Bytes::new()
    } else {
        let offset = as_usize_or_fail!(interpreter, offset);
        resize_memory!(interpreter, offset, len);
        Bytes::copy_from_slice(interpreter.memory.slice_len(offset, len).as_ref())
    };
    if interpreter.stack.len() < N {
        interpreter
            .control
            .set_instruction_result(InstructionResult::StackUnderflow);
        return;
    }
    let Some(topics) = interpreter.stack.popn::<N>() else {
        interpreter
            .control
            .set_instruction_result(InstructionResult::StackUnderflow);
        return;
    };

    let log = Log {
        address: interpreter.input.target_address(),
        data: LogData::new(topics.into_iter().map(B256::from).collect(), data)
            .expect("LogData should have <=4 topics"),
    };

    host.log(log);
}

pub fn selfdestruct<WIRE: InterpreterTypes, H: Host + ?Sized>(
    interpreter: &mut Interpreter<WIRE>,
    host: &mut H,
) {
    require_non_staticcall!(interpreter);
    popn!([target], interpreter);
    let target = target.into_address();
    let Some(res) = host.selfdestruct(interpreter.input.target_address(), target) else {
        interpreter
            .control
            .set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };

    // EIP-3529: Reduction in refunds
    if !interpreter.runtime_flag.spec_id().is_enabled_in(LONDON) && !res.previously_destroyed {
        interpreter.control.gas().record_refund(gas::SELFDESTRUCT)
    }
    gas!(
        interpreter,
        gas::selfdestruct_cost(interpreter.runtime_flag.spec_id(), res)
    );

    interpreter
        .control
        .set_instruction_result(InstructionResult::SelfDestruct);
}
