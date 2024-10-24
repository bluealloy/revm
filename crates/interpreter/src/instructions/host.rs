use crate::{
    gas::{self, warm_cold_cost, warm_cold_cost_with_delegation, CALL_STIPEND},
    interpreter::{Interpreter, InterpreterTrait},
    Host, InstructionResult,
};
use core::cmp::min;
use primitives::{Bytes, Log, LogData, B256, U256};
use specification::hardfork::{Spec, SpecId::*};
use std::vec::Vec;

pub fn balance<I: InterpreterTrait, H: Host + ?Sized>(interpreter: &mut I, host: &mut H) {
    pop_address!(interpreter, address);
    let Some(balance) = host.balance(address) else {
        interpreter.set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };
    gas!(
        interpreter,
        if interpreter.spec_id().is_enabled_in(BERLIN) {
            warm_cold_cost(balance.is_cold)
        } else if interpreter.spec_id().is_enabled_in(ISTANBUL) {
            // EIP-1884: Repricing for trie-size-dependent opcodes
            700
        } else if interpreter.spec_id().is_enabled_in(TANGERINE) {
            400
        } else {
            20
        }
    );
    push!(interpreter, balance.data);
}

/// EIP-1884: Repricing for trie-size-dependent opcodes
pub fn selfbalance<I: InterpreterTrait, H: Host + ?Sized>(interpreter: &mut I, host: &mut H) {
    check!(interpreter, ISTANBUL);
    gas!(interpreter, gas::LOW);
    let Some(balance) = host.balance(interpreter.target_address()) else {
        interpreter.set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };
    push!(interpreter, balance.data);
}

pub fn extcodesize<I: InterpreterTrait, H: Host + ?Sized>(interpreter: &mut I, host: &mut H) {
    pop_address!(interpreter, address);
    let Some(code) = host.code(address) else {
        interpreter.set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };
    let (code, load) = code.into_components();
    if interpreter.spec_id().is_enabled_in(BERLIN) {
        gas!(interpreter, warm_cold_cost_with_delegation(load));
    } else if interpreter.spec_id().is_enabled_in(TANGERINE) {
        gas!(interpreter, 700);
    } else {
        gas!(interpreter, 20);
    }

    push!(interpreter, U256::from(code.len()));
}

/// EIP-1052: EXTCODEHASH opcode
pub fn extcodehash<I: InterpreterTrait, H: Host + ?Sized>(interpreter: &mut I, host: &mut H) {
    check!(interpreter, CONSTANTINOPLE);
    pop_address!(interpreter, address);
    let Some(code_hash) = host.code_hash(address) else {
        interpreter.set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };
    let (code_hash, load) = code_hash.into_components();
    if interpreter.spec_id().is_enabled_in(BERLIN) {
        gas!(interpreter, warm_cold_cost_with_delegation(load))
    } else if interpreter.spec_id().is_enabled_in(ISTANBUL) {
        gas!(interpreter, 700);
    } else {
        gas!(interpreter, 400);
    }
    push_b256!(interpreter, code_hash);
}

pub fn extcodecopy<I: InterpreterTrait, H: Host + ?Sized>(interpreter: &mut I, host: &mut H) {
    pop_address!(interpreter, address);
    pop!(interpreter, memory_offset, code_offset, len_u256);

    let Some(code) = host.code(address) else {
        interpreter.set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };

    let len = as_usize_or_fail!(interpreter, len_u256);
    let (code, load) = code.into_components();
    gas_or_fail!(
        interpreter,
        gas::extcodecopy_cost(interpreter.spec_id(), len as u64, load)
    );
    if len == 0 {
        return;
    }
    let memory_offset = as_usize_or_fail!(interpreter, memory_offset);
    let code_offset = min(as_usize_saturated!(code_offset), code.len());
    resize_memory!(interpreter, memory_offset, len);

    // Note: this can't panic because we resized memory to fit.
    interpreter.mem_set_data(memory_offset, code_offset, len, &code);
}

pub fn blockhash<I: InterpreterTrait, H: Host + ?Sized>(interpreter: &mut I, host: &mut H) {
    gas!(interpreter, gas::BLOCKHASH);
    pop_top!(interpreter, number);

    let number_u64 = as_u64_saturated!(number);
    let Some(hash) = host.block_hash(number_u64) else {
        interpreter.set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };
    *number = U256::from_be_bytes(hash.0);
}

pub fn sload<I: InterpreterTrait, H: Host + ?Sized>(interpreter: &mut I, host: &mut H) {
    pop_top!(interpreter, index);
    let Some(value) = host.sload(interpreter.target_address(), *index) else {
        interpreter.set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };
    gas!(
        interpreter,
        gas::sload_cost(interpreter.spec_id(), value.is_cold)
    );
    *index = value.data;
}

pub fn sstore<I: InterpreterTrait, H: Host + ?Sized>(interpreter: &mut I, host: &mut H) {
    require_non_staticcall!(interpreter);

    pop!(interpreter, index, value);
    let Some(state_load) = host.sstore(interpreter.target_address(), index, value) else {
        interpreter.set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };

    // EIP-1706 Disable SSTORE with gasleft lower than call stipend
    if interpreter.spec_id().is_enabled_in(ISTANBUL)
        && interpreter.gas().remaining() <= CALL_STIPEND
    {
        interpreter.set_instruction_result(InstructionResult::ReentrancySentryOOG);
        return;
    }
    gas!(
        interpreter,
        gas::sstore_cost(interpreter.spec_id(), &state_load.data, state_load.is_cold)
    );
    refund!(
        interpreter,
        gas::sstore_refund(interpreter.spec_id(), &state_load.data)
    );
}

/// EIP-1153: Transient storage opcodes
/// Store value to transient storage
pub fn tstore<I: InterpreterTrait, H: Host + ?Sized>(interpreter: &mut I, host: &mut H) {
    check!(interpreter, CANCUN);
    require_non_staticcall!(interpreter);
    gas!(interpreter, gas::WARM_STORAGE_READ_COST);

    pop!(interpreter, index, value);

    host.tstore(interpreter.target_address(), index, value);
}

/// EIP-1153: Transient storage opcodes
/// Load value from transient storage
pub fn tload<I: InterpreterTrait, H: Host + ?Sized>(interpreter: &mut I, host: &mut H) {
    check!(interpreter, CANCUN);
    gas!(interpreter, gas::WARM_STORAGE_READ_COST);

    pop_top!(interpreter, index);

    *index = host.tload(interpreter.target_address(), *index);
}

pub fn log<const N: usize, H: Host + ?Sized>(
    interpreter: &mut impl InterpreterTrait,
    host: &mut H,
) {
    require_non_staticcall!(interpreter);

    pop!(interpreter, offset, len);
    let len = as_usize_or_fail!(interpreter, len);
    gas_or_fail!(interpreter, gas::log_cost(N as u8, len as u64));
    let data = if len == 0 {
        Bytes::new()
    } else {
        let offset = as_usize_or_fail!(interpreter, offset);
        resize_memory!(interpreter, offset, len);
        Bytes::copy_from_slice(interpreter.mem_slice_len(offset, len))
    };

    let topics = match N {
        0 => vec![],
        1 => vec![otry!(interpreter.pop())],
        2 => otry!(interpreter.pop2()).to_vec(),
        3 => otry!(interpreter.pop3()).to_vec(),
        4 => otry!(interpreter.pop4()).to_vec(),
        _ => unreachable!("LogData should have <=4 topics"),
    };

    let log = Log {
        address: interpreter.target_address(),
        data: LogData::new(topics.into_iter().map(B256::from).collect(), data)
            .expect("LogData should have <=4 topics"),
    };

    host.log(log);
}

pub fn selfdestruct<I: InterpreterTrait, H: Host + ?Sized>(interpreter: &mut I, host: &mut H) {
    require_non_staticcall!(interpreter);
    pop_address!(interpreter, target);

    let Some(res) = host.selfdestruct(interpreter.target_address(), target) else {
        interpreter.set_instruction_result(InstructionResult::FatalExternalError);
        return;
    };

    // EIP-3529: Reduction in refunds
    if !interpreter.spec_id().is_enabled_in(LONDON) && !res.previously_destroyed {
        refund!(interpreter, gas::SELFDESTRUCT)
    }
    gas!(
        interpreter,
        gas::selfdestruct_cost(interpreter.spec_id(), res)
    );

    interpreter.set_instruction_result(InstructionResult::SelfDestruct);
}
