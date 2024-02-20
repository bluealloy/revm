use crate::{
    gas::{self, COLD_ACCOUNT_ACCESS_COST, WARM_STORAGE_READ_COST},
    interpreter::{Interpreter, InterpreterAction},
    primitives::{Address, Bytes, Log, LogData, Spec, SpecId::*, B256, U256},
    CallContext, CallInputs, CallScheme, CreateInputs, CreateScheme, Host, InstructionResult,
    Transfer, MAX_INITCODE_SIZE,
};
use alloc::{boxed::Box, vec::Vec};
use core::cmp::min;
use revm_primitives::BLOCK_HASH_HISTORY;

pub fn balance<H: Host, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    pop_address!(interpreter, address);
    let Some((balance, is_cold)) = host.balance(address) else {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    };
    gas!(
        interpreter,
        if SPEC::enabled(ISTANBUL) {
            // EIP-1884: Repricing for trie-size-dependent opcodes
            gas::account_access_gas::<SPEC>(is_cold)
        } else if SPEC::enabled(TANGERINE) {
            400
        } else {
            20
        }
    );
    push!(interpreter, balance);
}

/// EIP-1884: Repricing for trie-size-dependent opcodes
pub fn selfbalance<H: Host, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    check!(interpreter, ISTANBUL);
    gas!(interpreter, gas::LOW);
    let Some((balance, _)) = host.balance(interpreter.contract.address) else {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    };
    push!(interpreter, balance);
}

pub fn extcodesize<H: Host, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    panic_on_eof!(interpreter);
    pop_address!(interpreter, address);
    let Some((code, is_cold)) = host.code(address) else {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    };
    if SPEC::enabled(BERLIN) {
        gas!(
            interpreter,
            if is_cold {
                COLD_ACCOUNT_ACCESS_COST
            } else {
                WARM_STORAGE_READ_COST
            }
        );
    } else if SPEC::enabled(TANGERINE) {
        gas!(interpreter, 700);
    } else {
        gas!(interpreter, 20);
    }

    push!(interpreter, U256::from(code.len()));
}

/// EIP-1052: EXTCODEHASH opcode
pub fn extcodehash<H: Host, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    panic_on_eof!(interpreter);
    check!(interpreter, CONSTANTINOPLE);
    pop_address!(interpreter, address);
    let Some((code_hash, is_cold)) = host.code_hash(address) else {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    };
    if SPEC::enabled(BERLIN) {
        gas!(
            interpreter,
            if is_cold {
                COLD_ACCOUNT_ACCESS_COST
            } else {
                WARM_STORAGE_READ_COST
            }
        );
    } else if SPEC::enabled(ISTANBUL) {
        gas!(interpreter, 700);
    } else {
        gas!(interpreter, 400);
    }
    push_b256!(interpreter, code_hash);
}

pub fn extcodecopy<H: Host, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    panic_on_eof!(interpreter);
    pop_address!(interpreter, address);
    pop!(interpreter, memory_offset, code_offset, len_u256);

    let Some((code, is_cold)) = host.code(address) else {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    };

    let len = as_usize_or_fail!(interpreter, len_u256);
    gas_or_fail!(
        interpreter,
        gas::extcodecopy_cost::<SPEC>(len as u64, is_cold)
    );
    if len == 0 {
        return;
    }
    let memory_offset = as_usize_or_fail!(interpreter, memory_offset);
    let code_offset = min(as_usize_saturated!(code_offset), code.len());
    shared_memory_resize!(interpreter, memory_offset, len);

    // Note: this can't panic because we resized memory to fit.
    interpreter
        .shared_memory
        .set_data(memory_offset, code_offset, len, code.bytes());
}

pub fn blockhash<H: Host>(interpreter: &mut Interpreter, host: &mut H) {
    gas!(interpreter, gas::BLOCKHASH);
    pop_top!(interpreter, number);

    if let Some(diff) = host.env().block.number.checked_sub(*number) {
        let diff = as_usize_saturated!(diff);
        // blockhash should push zero if number is same as current block number.
        if diff <= BLOCK_HASH_HISTORY && diff != 0 {
            let Some(hash) = host.block_hash(*number) else {
                interpreter.instruction_result = InstructionResult::FatalExternalError;
                return;
            };
            *number = U256::from_be_bytes(hash.0);
            return;
        }
    }
    *number = U256::ZERO;
}

pub fn sload<H: Host, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    pop!(interpreter, index);

    let Some((value, is_cold)) = host.sload(interpreter.contract.address, index) else {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    };
    gas!(interpreter, gas::sload_cost::<SPEC>(is_cold));
    push!(interpreter, value);
}

pub fn sstore<H: Host, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    error_on_static_call!(interpreter);

    pop!(interpreter, index, value);
    let Some((original, old, new, is_cold)) =
        host.sstore(interpreter.contract.address, index, value)
    else {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    };
    gas_or_fail!(interpreter, {
        let remaining_gas = interpreter.gas.remaining();
        gas::sstore_cost::<SPEC>(original, old, new, remaining_gas, is_cold)
    });
    refund!(interpreter, gas::sstore_refund::<SPEC>(original, old, new));
}

/// EIP-1153: Transient storage opcodes
/// Store value to transient storage
pub fn tstore<H: Host, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    check!(interpreter, CANCUN);
    error_on_static_call!(interpreter);
    gas!(interpreter, gas::WARM_STORAGE_READ_COST);

    pop!(interpreter, index, value);

    host.tstore(interpreter.contract.address, index, value);
}

/// EIP-1153: Transient storage opcodes
/// Load value from transient storage
pub fn tload<H: Host, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    check!(interpreter, CANCUN);
    gas!(interpreter, gas::WARM_STORAGE_READ_COST);

    pop_top!(interpreter, index);

    *index = host.tload(interpreter.contract.address, *index);
}

pub fn log<const N: usize, H: Host>(interpreter: &mut Interpreter, host: &mut H) {
    error_on_static_call!(interpreter);

    pop!(interpreter, offset, len);
    let len = as_usize_or_fail!(interpreter, len);
    gas_or_fail!(interpreter, gas::log_cost(N as u8, len as u64));
    let data = if len == 0 {
        Bytes::new()
    } else {
        let offset = as_usize_or_fail!(interpreter, offset);
        shared_memory_resize!(interpreter, offset, len);
        Bytes::copy_from_slice(interpreter.shared_memory.slice(offset, len))
    };

    if interpreter.stack.len() < N {
        interpreter.instruction_result = InstructionResult::StackUnderflow;
        return;
    }

    let mut topics = Vec::with_capacity(N);
    for _ in 0..N {
        // SAFETY: stack bounds already checked few lines above
        topics.push(B256::from(unsafe { interpreter.stack.pop_unsafe() }));
    }

    let log = Log {
        address: interpreter.contract.address,
        data: LogData::new(topics, data).expect("LogData should have <=4 topics"),
    };

    host.log(log);
}

pub fn selfdestruct<H: Host, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    panic_on_eof!(interpreter);
    error_on_static_call!(interpreter);
    pop_address!(interpreter, target);

    let Some(res) = host.selfdestruct(interpreter.contract.address, target) else {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    };

    // EIP-3529: Reduction in refunds
    if !SPEC::enabled(LONDON) && !res.previously_destroyed {
        refund!(interpreter, gas::SELFDESTRUCT)
    }
    gas!(interpreter, gas::selfdestruct_cost::<SPEC>(res));

    interpreter.instruction_result = InstructionResult::SelfDestruct;
}
