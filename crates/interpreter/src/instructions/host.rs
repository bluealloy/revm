use crate::{
    gas::{self, COLD_ACCOUNT_ACCESS_COST, WARM_STORAGE_READ_COST},
    interpreter::Interpreter,
    primitives::{Bytes, Spec, SpecId::*, B160, B256, U256},
    return_ok, return_revert, CallContext, CallInputs, CallScheme, CreateInputs, CreateScheme,
    Host, InstructionResult, Transfer, MAX_INITCODE_SIZE,
};
use alloc::{boxed::Box, vec::Vec};
use core::cmp::min;
use revm_primitives::BLOCK_HASH_HISTORY;

pub fn balance<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
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
pub fn selfbalance<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    check!(interpreter, ISTANBUL);
    gas!(interpreter, gas::LOW);
    let Some((balance, _)) = host.balance(interpreter.contract.address) else {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    };
    push!(interpreter, balance);
}

pub fn extcodesize<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
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
pub fn extcodehash<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
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

pub fn extcodecopy<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
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
    memory_resize!(interpreter, memory_offset, len);

    // Safety: set_data is unsafe function and memory_resize ensures us that it is safe to call it
    interpreter
        .memory
        .set_data(memory_offset, code_offset, len, code.bytes());
}

pub fn blockhash(interpreter: &mut Interpreter, host: &mut dyn Host) {
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

pub fn sload<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    pop!(interpreter, index);

    let Some((value, is_cold)) = host.sload(interpreter.contract.address, index) else {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    };
    gas!(interpreter, gas::sload_cost::<SPEC>(is_cold));
    push!(interpreter, value);
}

pub fn sstore<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
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
pub fn tstore<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    check!(interpreter, CANCUN);
    check_staticcall!(interpreter);
    gas!(interpreter, gas::WARM_STORAGE_READ_COST);

    pop!(interpreter, index, value);

    host.tstore(interpreter.contract.address, index, value);
}

/// EIP-1153: Transient storage opcodes
/// Load value from transient storage
pub fn tload<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    check!(interpreter, CANCUN);
    gas!(interpreter, gas::WARM_STORAGE_READ_COST);

    pop_top!(interpreter, index);

    *index = host.tload(interpreter.contract.address, *index);
}

pub fn log<const N: usize>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    check_staticcall!(interpreter);

    pop!(interpreter, offset, len);
    let len = as_usize_or_fail!(interpreter, len);
    gas_or_fail!(interpreter, gas::log_cost(N as u8, len as u64));
    let data = if len == 0 {
        Bytes::new()
    } else {
        let offset = as_usize_or_fail!(interpreter, offset);
        memory_resize!(interpreter, offset, len);
        Bytes::copy_from_slice(interpreter.memory.slice(offset, len))
    };

    if interpreter.stack.len() < N {
        interpreter.instruction_result = InstructionResult::StackUnderflow;
        return;
    }

    let mut topics = Vec::with_capacity(N);
    for _ in 0..N {
        // Safety: stack bounds already checked few lines above
        topics.push(B256(unsafe {
            interpreter.stack.pop_unsafe().to_be_bytes()
        }));
    }

    host.log(interpreter.contract.address, topics, data);
}

pub fn selfdestruct<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
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

#[inline(never)]
pub fn prepare_create_inputs<const IS_CREATE2: bool, SPEC: Spec>(
    interpreter: &mut Interpreter,
    host: &mut dyn Host,
    create_inputs: &mut Option<Box<CreateInputs>>,
) {
    check_staticcall!(interpreter);

    // EIP-1014: Skinny CREATE2
    if IS_CREATE2 {
        check!(interpreter, PETERSBURG);
    }

    interpreter.return_data_buffer = Bytes::new();

    pop!(interpreter, value, code_offset, len);
    let len = as_usize_or_fail!(interpreter, len);

    let code = if len == 0 {
        Bytes::new()
    } else {
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
                interpreter.instruction_result = InstructionResult::CreateInitcodeSizeLimit;
                return;
            }
            gas!(interpreter, gas::initcode_cost(len as u64));
        }

        let code_offset = as_usize_or_fail!(interpreter, code_offset);
        memory_resize!(interpreter, code_offset, len);
        Bytes::copy_from_slice(interpreter.memory.slice(code_offset, len))
    };

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

    *create_inputs = Some(Box::new(CreateInputs {
        caller: interpreter.contract.address,
        scheme,
        value,
        init_code: code,
        gas_limit,
    }));
}

pub fn create<const IS_CREATE2: bool, SPEC: Spec>(
    interpreter: &mut Interpreter,
    host: &mut dyn Host,
) {
    let mut create_input: Option<Box<CreateInputs>> = None;
    prepare_create_inputs::<IS_CREATE2, SPEC>(interpreter, host, &mut create_input);

    let Some(mut create_input) = create_input else {
        return;
    };

    let (return_reason, address, gas, return_data) = host.create(&mut create_input);

    interpreter.return_data_buffer = match return_reason {
        // Save data to return data buffer if the create reverted
        return_revert!() => return_data,
        // Otherwise clear it
        _ => Bytes::new(),
    };

    match return_reason {
        return_ok!() => {
            push_b256!(interpreter, address.map_or(B256::zero(), |a| a.into()));
            if crate::USE_GAS {
                interpreter.gas.erase_cost(gas.remaining());
                interpreter.gas.record_refund(gas.refunded());
            }
        }
        return_revert!() => {
            push!(interpreter, U256::ZERO);
            if crate::USE_GAS {
                interpreter.gas.erase_cost(gas.remaining());
            }
        }
        InstructionResult::FatalExternalError => {
            interpreter.instruction_result = InstructionResult::FatalExternalError;
        }
        _ => push!(interpreter, U256::ZERO),
    }
}

pub fn call<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    call_inner::<SPEC>(CallScheme::Call, interpreter, host);
}

pub fn call_code<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    call_inner::<SPEC>(CallScheme::CallCode, interpreter, host);
}

pub fn delegate_call<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    call_inner::<SPEC>(CallScheme::DelegateCall, interpreter, host);
}

pub fn static_call<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    call_inner::<SPEC>(CallScheme::StaticCall, interpreter, host);
}

#[inline(never)]
fn prepare_call_inputs<SPEC: Spec>(
    interpreter: &mut Interpreter,
    scheme: CallScheme,
    host: &mut dyn Host,
    result_len: &mut usize,
    result_offset: &mut usize,
    result_call_inputs: &mut Option<Box<CallInputs>>,
) {
    pop!(interpreter, local_gas_limit);
    pop_address!(interpreter, to);
    let local_gas_limit = u64::try_from(local_gas_limit).unwrap_or(u64::MAX);

    let value = match scheme {
        CallScheme::CallCode => {
            pop!(interpreter, value);
            value
        }
        CallScheme::Call => {
            pop!(interpreter, value);
            if interpreter.is_static && value != U256::ZERO {
                interpreter.instruction_result = InstructionResult::CallNotAllowedInsideStatic;
                return;
            }
            value
        }
        CallScheme::DelegateCall | CallScheme::StaticCall => U256::ZERO,
    };

    pop!(interpreter, in_offset, in_len, out_offset, out_len);

    let in_len = as_usize_or_fail!(interpreter, in_len);
    let input = if in_len != 0 {
        let in_offset = as_usize_or_fail!(interpreter, in_offset);
        memory_resize!(interpreter, in_offset, in_len);
        Bytes::copy_from_slice(interpreter.memory.slice(in_offset, in_len))
    } else {
        Bytes::new()
    };

    *result_len = as_usize_or_fail!(interpreter, out_len);
    *result_offset = if *result_len != 0 {
        let out_offset = as_usize_or_fail!(interpreter, out_offset);
        memory_resize!(interpreter, out_offset, *result_len);
        out_offset
    } else {
        usize::MAX //unrealistic value so we are sure it is not used
    };

    let context = match scheme {
        CallScheme::Call | CallScheme::StaticCall => CallContext {
            address: to,
            caller: interpreter.contract.address,
            code_address: to,
            apparent_value: value,
            scheme,
        },
        CallScheme::CallCode => CallContext {
            address: interpreter.contract.address,
            caller: interpreter.contract.address,
            code_address: to,
            apparent_value: value,
            scheme,
        },
        CallScheme::DelegateCall => CallContext {
            address: interpreter.contract.address,
            caller: interpreter.contract.caller,
            code_address: to,
            apparent_value: interpreter.contract.value,
            scheme,
        },
    };

    let transfer = if scheme == CallScheme::Call {
        Transfer {
            source: interpreter.contract.address,
            target: to,
            value,
        }
    } else if scheme == CallScheme::CallCode {
        Transfer {
            source: interpreter.contract.address,
            target: interpreter.contract.address,
            value,
        }
    } else {
        //this is dummy send for StaticCall and DelegateCall, it should do nothing and dont touch anything.
        Transfer {
            source: interpreter.contract.address,
            target: interpreter.contract.address,
            value: U256::ZERO,
        }
    };

    // load account and calculate gas cost.
    let Some((is_cold, exist)) = host.load_account(to) else {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    };
    let is_new = !exist;

    gas!(
        interpreter,
        gas::call_cost::<SPEC>(
            value,
            is_new,
            is_cold,
            matches!(scheme, CallScheme::Call | CallScheme::CallCode),
            matches!(scheme, CallScheme::Call | CallScheme::StaticCall),
        )
    );

    // EIP-150: Gas cost changes for IO-heavy operations
    let mut gas_limit = if SPEC::enabled(TANGERINE) {
        let gas = interpreter.gas().remaining();
        // take l64 part of gas_limit
        min(gas - gas / 64, local_gas_limit)
    } else {
        local_gas_limit
    };

    gas!(interpreter, gas_limit);

    // add call stipend if there is value to be transferred.
    if matches!(scheme, CallScheme::Call | CallScheme::CallCode) && transfer.value != U256::ZERO {
        gas_limit = gas_limit.saturating_add(gas::CALL_STIPEND);
    }
    let is_static = matches!(scheme, CallScheme::StaticCall) || interpreter.is_static;

    *result_call_inputs = Some(Box::new(CallInputs {
        contract: to,
        transfer,
        input,
        gas_limit,
        context,
        is_static,
    }));
}

pub fn call_inner<SPEC: Spec>(
    scheme: CallScheme,
    interpreter: &mut Interpreter,
    host: &mut dyn Host,
) {
    match scheme {
        // EIP-7: DELEGATECALL
        CallScheme::DelegateCall => check!(interpreter, HOMESTEAD),
        // EIP-214: New opcode STATICCALL
        CallScheme::StaticCall => check!(interpreter, BYZANTIUM),
        _ => (),
    }
    interpreter.return_data_buffer = Bytes::new();

    let mut out_offset: usize = 0;
    let mut out_len: usize = 0;
    let mut call_input: Option<Box<CallInputs>> = None;
    prepare_call_inputs::<SPEC>(
        interpreter,
        scheme,
        host,
        &mut out_len,
        &mut out_offset,
        &mut call_input,
    );

    let Some(mut call_input) = call_input else {
        return;
    };

    // Call host to interact with target contract
    let (reason, gas, return_data) = host.call(&mut call_input);

    interpreter.return_data_buffer = return_data;

    let target_len = min(out_len, interpreter.return_data_buffer.len());

    match reason {
        return_ok!() => {
            // return unspend gas.
            if crate::USE_GAS {
                interpreter.gas.erase_cost(gas.remaining());
                interpreter.gas.record_refund(gas.refunded());
            }
            interpreter
                .memory
                .set(out_offset, &interpreter.return_data_buffer[..target_len]);
            push!(interpreter, U256::from(1));
        }
        return_revert!() => {
            if crate::USE_GAS {
                interpreter.gas.erase_cost(gas.remaining());
            }
            interpreter
                .memory
                .set(out_offset, &interpreter.return_data_buffer[..target_len]);
            push!(interpreter, U256::ZERO);
        }
        InstructionResult::FatalExternalError => {
            interpreter.instruction_result = InstructionResult::FatalExternalError;
        }
        _ => {
            push!(interpreter, U256::ZERO);
        }
    }
}
