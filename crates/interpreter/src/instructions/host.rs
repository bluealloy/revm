mod call_helpers;

pub use call_helpers::{calc_call_gas, get_memory_input_and_out_ranges};

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
    check_staticcall!(interpreter);

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
    check_staticcall!(interpreter);
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
    check_staticcall!(interpreter);

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
    check_staticcall!(interpreter);
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

pub fn create<const IS_CREATE2: bool, H: Host, SPEC: Spec>(
    interpreter: &mut Interpreter,
    host: &mut H,
) {
    check_staticcall!(interpreter);

    // EIP-1014: Skinny CREATE2
    if IS_CREATE2 {
        check!(interpreter, PETERSBURG);
    }

    pop!(interpreter, value, code_offset, len);
    let len = as_usize_or_fail!(interpreter, len);

    let mut code = Bytes::new();
    if len != 0 {
        // EIP-3860: Limit and meter initcode
        if SPEC::enabled(SHANGHAI) {
            // Limit is set as double of max contract bytecode size
            let max_initcode_size = host
                .env()
                .cfg
                .limit_contract_code_size
                .map(|limit| limit.saturating_mul(2))
                .unwrap_or(MAX_INITCODE_SIZE);
            if len > max_initcode_size {
                interpreter.instruction_result = InstructionResult::CreateInitCodeSizeLimit;
                return;
            }
            gas!(interpreter, gas::initcode_cost(len as u64));
        }

        let code_offset = as_usize_or_fail!(interpreter, code_offset);
        shared_memory_resize!(interpreter, code_offset, len);
        code = Bytes::copy_from_slice(interpreter.shared_memory.slice(code_offset, len));
    }

    // EIP-1014: Skinny CREATE2
    let scheme = if IS_CREATE2 {
        pop!(interpreter, salt);
        gas_or_fail!(interpreter, gas::create2_cost(len));
        CreateScheme::Create2 { salt }
    } else {
        gas!(interpreter, gas::CREATE);
        CreateScheme::Create
    };

    let mut gas_limit = interpreter.gas().remaining();

    // EIP-150: Gas cost changes for IO-heavy operations
    if SPEC::enabled(TANGERINE) {
        // take remaining gas and deduce l64 part of it.
        gas_limit -= gas_limit / 64
    }
    gas!(interpreter, gas_limit);

    // Call host to interact with target contract
    interpreter.next_action = Some(InterpreterAction::Create {
        inputs: Box::new(CreateInputs {
            caller: interpreter.contract.address,
            scheme,
            value,
            init_code: code,
            gas_limit,
        }),
    });
    interpreter.instruction_result = InstructionResult::CallOrCreate;
}

pub fn call<H: Host, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    pop!(interpreter, local_gas_limit);
    pop_address!(interpreter, to);
    // max gas limit is not possible in real ethereum situation.
    let local_gas_limit = u64::try_from(local_gas_limit).unwrap_or(u64::MAX);

    pop!(interpreter, value);
    if interpreter.is_static && value != U256::ZERO {
        interpreter.instruction_result = InstructionResult::CallNotAllowedInsideStatic;
        return;
    }

    let Some((input, return_memory_offset)) = get_memory_input_and_out_ranges(interpreter) else {
        return;
    };

    let Some(mut gas_limit) = calc_call_gas::<H, SPEC>(
        interpreter,
        host,
        to,
        value != U256::ZERO,
        local_gas_limit,
        true,
        true,
    ) else {
        return;
    };

    gas!(interpreter, gas_limit);

    // add call stipend if there is value to be transferred.
    if value != U256::ZERO {
        gas_limit = gas_limit.saturating_add(gas::CALL_STIPEND);
    }

    // Call host to interact with target contract
    interpreter.next_action = Some(InterpreterAction::SubCall {
        inputs: Box::new(CallInputs {
            contract: to,
            transfer: Transfer {
                source: interpreter.contract.address,
                target: to,
                value,
            },
            input,
            gas_limit,
            context: CallContext {
                address: to,
                caller: interpreter.contract.address,
                code_address: to,
                apparent_value: value,
                scheme: CallScheme::Call,
            },
            is_static: interpreter.is_static,
        }),
        return_memory_offset,
    });
    interpreter.instruction_result = InstructionResult::CallOrCreate;
}

pub fn call_code<H: Host, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    pop!(interpreter, local_gas_limit);
    pop_address!(interpreter, to);
    // max gas limit is not possible in real ethereum situation.
    let local_gas_limit = u64::try_from(local_gas_limit).unwrap_or(u64::MAX);

    pop!(interpreter, value);
    let Some((input, return_memory_offset)) = get_memory_input_and_out_ranges(interpreter) else {
        return;
    };

    let Some(mut gas_limit) = calc_call_gas::<H, SPEC>(
        interpreter,
        host,
        to,
        value != U256::ZERO,
        local_gas_limit,
        true,
        false,
    ) else {
        return;
    };

    gas!(interpreter, gas_limit);

    // add call stipend if there is value to be transferred.
    if value != U256::ZERO {
        gas_limit = gas_limit.saturating_add(gas::CALL_STIPEND);
    }

    // Call host to interact with target contract
    interpreter.next_action = Some(InterpreterAction::SubCall {
        inputs: Box::new(CallInputs {
            contract: to,
            transfer: Transfer {
                source: interpreter.contract.address,
                target: interpreter.contract.address,
                value,
            },
            input,
            gas_limit,
            context: CallContext {
                address: interpreter.contract.address,
                caller: interpreter.contract.address,
                code_address: to,
                apparent_value: value,
                scheme: CallScheme::CallCode,
            },
            is_static: interpreter.is_static,
        }),
        return_memory_offset,
    });
    interpreter.instruction_result = InstructionResult::CallOrCreate;
}

pub fn delegate_call<H: Host, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    check!(interpreter, HOMESTEAD);
    pop!(interpreter, local_gas_limit);
    pop_address!(interpreter, to);
    // max gas limit is not possible in real ethereum situation.
    let local_gas_limit = u64::try_from(local_gas_limit).unwrap_or(u64::MAX);

    let Some((input, return_memory_offset)) = get_memory_input_and_out_ranges(interpreter) else {
        return;
    };

    let Some(gas_limit) =
        calc_call_gas::<H, SPEC>(interpreter, host, to, false, local_gas_limit, false, false)
    else {
        return;
    };

    gas!(interpreter, gas_limit);

    // Call host to interact with target contract
    interpreter.next_action = Some(InterpreterAction::SubCall {
        inputs: Box::new(CallInputs {
            contract: to,
            // This is dummy send for StaticCall and DelegateCall,
            // it should do nothing and not touch anything.
            transfer: Transfer {
                source: interpreter.contract.address,
                target: interpreter.contract.address,
                value: U256::ZERO,
            },
            input,
            gas_limit,
            context: CallContext {
                address: interpreter.contract.address,
                caller: interpreter.contract.caller,
                code_address: to,
                apparent_value: interpreter.contract.value,
                scheme: CallScheme::DelegateCall,
            },
            is_static: interpreter.is_static,
        }),
        return_memory_offset,
    });
    interpreter.instruction_result = InstructionResult::CallOrCreate;
}

pub fn static_call<H: Host, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    check!(interpreter, BYZANTIUM);
    pop!(interpreter, local_gas_limit);
    pop_address!(interpreter, to);
    // max gas limit is not possible in real ethereum situation.
    let local_gas_limit = u64::try_from(local_gas_limit).unwrap_or(u64::MAX);

    let value = U256::ZERO;
    let Some((input, return_memory_offset)) = get_memory_input_and_out_ranges(interpreter) else {
        return;
    };

    let Some(gas_limit) =
        calc_call_gas::<H, SPEC>(interpreter, host, to, false, local_gas_limit, false, true)
    else {
        return;
    };
    gas!(interpreter, gas_limit);

    // Call host to interact with target contract
    interpreter.next_action = Some(InterpreterAction::SubCall {
        inputs: Box::new(CallInputs {
            contract: to,
            // This is dummy send for StaticCall and DelegateCall,
            // it should do nothing and not touch anything.
            transfer: Transfer {
                source: interpreter.contract.address,
                target: interpreter.contract.address,
                value: U256::ZERO,
            },
            input,
            gas_limit,
            context: CallContext {
                address: to,
                caller: interpreter.contract.address,
                code_address: to,
                apparent_value: value,
                scheme: CallScheme::StaticCall,
            },
            is_static: true,
        }),
        return_memory_offset,
    });
    interpreter.instruction_result = InstructionResult::CallOrCreate;
}
