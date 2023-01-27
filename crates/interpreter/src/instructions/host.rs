use crate::primitives::{Spec, SpecId::*, B160, B256, U256};
use crate::{
    alloc::vec::Vec,
    gas::{self, COLD_ACCOUNT_ACCESS_COST, WARM_STORAGE_READ_COST},
    interpreter::Interpreter,
    return_ok, return_revert, CallContext, CallInputs, CallScheme, CreateInputs, CreateScheme,
    Host, InstructionResult, Transfer,
};
use bytes::Bytes;
use core::cmp::min;

pub fn balance<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    pop_address!(interpreter, address);
    let ret = host.balance(address);
    if ret.is_none() {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    }
    let (balance, is_cold) = ret.unwrap();
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

pub fn selfbalance<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    // gas!(interp, gas::LOW);
    // EIP-1884: Repricing for trie-size-dependent opcodes
    check!(interpreter, SPEC::enabled(ISTANBUL));
    let ret = host.balance(interpreter.contract.address);
    if ret.is_none() {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    }
    let (balance, _) = ret.unwrap();
    push!(interpreter, balance);
}

pub fn extcodesize<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    pop_address!(interpreter, address);
    let ret = host.code(address);
    if ret.is_none() {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    }
    let (code, is_cold) = ret.unwrap();
    if SPEC::enabled(BERLIN) && is_cold {
        // WARM_STORAGE_READ_COST is already calculated in gas block
        gas!(
            interpreter,
            COLD_ACCOUNT_ACCESS_COST - WARM_STORAGE_READ_COST
        );
    }

    push!(interpreter, U256::from(code.len()));
}

pub fn extcodehash<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    check!(interpreter, SPEC::enabled(CONSTANTINOPLE)); // EIP-1052: EXTCODEHASH opcode
    pop_address!(interpreter, address);
    let ret = host.code_hash(address);
    if ret.is_none() {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    }
    let (code_hash, is_cold) = ret.unwrap();
    if SPEC::enabled(BERLIN) && is_cold {
        // WARM_STORAGE_READ_COST is already calculated in gas block
        gas!(
            interpreter,
            COLD_ACCOUNT_ACCESS_COST - WARM_STORAGE_READ_COST
        );
    }
    push_b256!(interpreter, code_hash);
}

pub fn extcodecopy<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    pop_address!(interpreter, address);
    pop!(interpreter, memory_offset, code_offset, len_u256);

    let ret = host.code(address);
    if ret.is_none() {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    }
    let (code, is_cold) = ret.unwrap();

    let len = as_usize_or_fail!(interpreter, len_u256, InstructionResult::OutOfGas);
    gas_or_fail!(
        interpreter,
        gas::extcodecopy_cost::<SPEC>(len as u64, is_cold)
    );
    if len == 0 {
        return;
    }
    let memory_offset = as_usize_or_fail!(interpreter, memory_offset, InstructionResult::OutOfGas);
    let code_offset = min(as_usize_saturated!(code_offset), code.len());
    memory_resize!(interpreter, memory_offset, len);

    // Safety: set_data is unsafe function and memory_resize ensures us that it is safe to call it
    interpreter
        .memory
        .set_data(memory_offset, code_offset, len, code.bytes());
}

pub fn blockhash(interpreter: &mut Interpreter, host: &mut dyn Host) {
    // gas!(interp, gas::BLOCKHASH);
    pop_top!(interpreter, number);

    if let Some(diff) = host.env().block.number.checked_sub(*number) {
        let diff = as_usize_saturated!(diff);
        // blockhash should push zero if number is same as current block number.
        if diff <= 256 && diff != 0 {
            let ret = host.block_hash(*number);
            if ret.is_none() {
                interpreter.instruction_result = InstructionResult::FatalExternalError;
                return;
            }
            *number = U256::from_be_bytes(*ret.unwrap());
            return;
        }
    }
    *number = U256::ZERO;
}

pub fn sload<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    pop!(interpreter, index);

    let ret = host.sload(interpreter.contract.address, index);
    if ret.is_none() {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    }
    let (value, is_cold) = ret.unwrap();
    gas!(interpreter, gas::sload_cost::<SPEC>(is_cold));
    push!(interpreter, value);
}

pub fn sstore<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    check!(interpreter, !interpreter.is_static);

    pop!(interpreter, index, value);
    let ret = host.sstore(interpreter.contract.address, index, value);
    if ret.is_none() {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    }
    let (original, old, new, is_cold) = ret.unwrap();
    gas_or_fail!(interpreter, {
        let remaining_gas = interpreter.gas.remaining();
        gas::sstore_cost::<SPEC>(original, old, new, remaining_gas, is_cold)
    });
    refund!(interpreter, gas::sstore_refund::<SPEC>(original, old, new));
    if let Some(ret) = interpreter.add_next_gas_block(interpreter.program_counter() - 1) {
        interpreter.instruction_result = ret;
    }
}

pub fn log<const N: u8, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    check!(interpreter, !interpreter.is_static);

    pop!(interpreter, offset, len);
    let len = as_usize_or_fail!(interpreter, len, InstructionResult::OutOfGas);
    gas_or_fail!(interpreter, gas::log_cost(N, len as u64));
    let data = if len == 0 {
        Bytes::new()
    } else {
        let offset = as_usize_or_fail!(interpreter, offset, InstructionResult::OutOfGas);
        memory_resize!(interpreter, offset, len);
        Bytes::copy_from_slice(interpreter.memory.get_slice(offset, len))
    };
    let n = N as usize;
    if interpreter.stack.len() < n {
        interpreter.instruction_result = InstructionResult::StackUnderflow;
        return;
    }

    let mut topics = Vec::with_capacity(n);
    for _ in 0..(n) {
        // Safety: stack bounds already checked few lines above
        topics.push(B256(unsafe {
            interpreter.stack.pop_unsafe().to_be_bytes()
        }));
    }

    host.log(interpreter.contract.address, topics, data);
}

pub fn selfdestruct<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    check!(interpreter, !interpreter.is_static);
    pop_address!(interpreter, target);

    let res = host.selfdestruct(interpreter.contract.address, target);
    if res.is_none() {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    }
    let res = res.unwrap();

    // EIP-3529: Reduction in refunds
    if !SPEC::enabled(LONDON) && !res.previously_destroyed {
        refund!(interpreter, gas::SELFDESTRUCT)
    }
    gas!(interpreter, gas::selfdestruct_cost::<SPEC>(res));

    interpreter.instruction_result = InstructionResult::SelfDestruct;
}

pub fn create<const IS_CREATE2: bool, SPEC: Spec>(
    interpreter: &mut Interpreter,
    host: &mut dyn Host,
) {
    check!(interpreter, !interpreter.is_static);
    if IS_CREATE2 {
        // EIP-1014: Skinny CREATE2
        check!(interpreter, SPEC::enabled(PETERSBURG));
    }

    interpreter.return_data_buffer = Bytes::new();

    pop!(interpreter, value, code_offset, len);
    let len = as_usize_or_fail!(interpreter, len, InstructionResult::OutOfGas);

    let code = if len == 0 {
        Bytes::new()
    } else {
        let code_offset = as_usize_or_fail!(interpreter, code_offset, InstructionResult::OutOfGas);
        memory_resize!(interpreter, code_offset, len);
        Bytes::copy_from_slice(interpreter.memory.get_slice(code_offset, len))
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

    let mut create_input = CreateInputs {
        caller: interpreter.contract.address,
        scheme,
        value,
        init_code: code,
        gas_limit,
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
            push_b256!(interpreter, address.unwrap_or_default().into());
            if crate::USE_GAS {
                interpreter.gas.erase_cost(gas.remaining());
                interpreter.gas.record_refund(gas.refunded());
            }
        }
        return_revert!() => {
            push_b256!(interpreter, B256::zero());
            if crate::USE_GAS {
                interpreter.gas.erase_cost(gas.remaining());
            }
        }
        InstructionResult::FatalExternalError => {
            interpreter.instruction_result = InstructionResult::FatalExternalError;
            return;
        }
        _ => {
            push_b256!(interpreter, B256::zero());
        }
    }
    if let Some(ret) = interpreter.add_next_gas_block(interpreter.program_counter() - 1) {
        interpreter.instruction_result = ret;
    }
}

pub fn call<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    call_inner::<SPEC>(interpreter, CallScheme::Call, host);
}

pub fn call_code<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    call_inner::<SPEC>(interpreter, CallScheme::CallCode, host);
}

pub fn delegate_call<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    call_inner::<SPEC>(interpreter, CallScheme::DelegateCall, host);
}

pub fn static_call<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    call_inner::<SPEC>(interpreter, CallScheme::StaticCall, host);
}

pub fn call_inner<SPEC: Spec>(
    interpreter: &mut Interpreter,
    scheme: CallScheme,
    host: &mut dyn Host,
) {
    match scheme {
        CallScheme::DelegateCall => check!(interpreter, SPEC::enabled(HOMESTEAD)), // EIP-7: DELEGATECALL
        CallScheme::StaticCall => check!(interpreter, SPEC::enabled(BYZANTIUM)), // EIP-214: New opcode STATICCALL
        _ => (),
    }
    interpreter.return_data_buffer = Bytes::new();

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

    let in_len = as_usize_or_fail!(interpreter, in_len, InstructionResult::OutOfGas);
    let input = if in_len != 0 {
        let in_offset = as_usize_or_fail!(interpreter, in_offset, InstructionResult::OutOfGas);
        memory_resize!(interpreter, in_offset, in_len);
        Bytes::copy_from_slice(interpreter.memory.get_slice(in_offset, in_len))
    } else {
        Bytes::new()
    };

    let out_len = as_usize_or_fail!(interpreter, out_len, InstructionResult::OutOfGas);
    let out_offset = if out_len != 0 {
        let out_offset = as_usize_or_fail!(interpreter, out_offset, InstructionResult::OutOfGas);
        memory_resize!(interpreter, out_offset, out_len);
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
    let res = host.load_account(to);
    if res.is_none() {
        interpreter.instruction_result = InstructionResult::FatalExternalError;
        return;
    }
    let (is_cold, exist) = res.unwrap();
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

    // take l64 part of gas_limit
    let mut gas_limit = if SPEC::enabled(TANGERINE) {
        //EIP-150: Gas cost changes for IO-heavy operations
        let gas = interpreter.gas().remaining();
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

    let mut call_input = CallInputs {
        contract: to,
        transfer,
        input,
        gas_limit,
        context,
        is_static,
    };

    // Call host to interuct with target contract
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
            return;
        }
        _ => {
            push!(interpreter, U256::ZERO);
        }
    }
    if let Some(ret) = interpreter.add_next_gas_block(interpreter.program_counter() - 1) {
        interpreter.instruction_result = ret;
    }
}
