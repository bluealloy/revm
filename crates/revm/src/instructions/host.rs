use crate::{
    alloc::vec::Vec, gas, interpreter::Interpreter, return_ok, return_revert, CallContext,
    CallInputs, CallScheme, CreateInputs, CreateScheme, Host, Return, Spec, SpecId::*, Transfer,
};
use bytes::Bytes;
use core::cmp::min;
use primitive_types::{H160, H256, U256};

pub fn balance<H: Host, SPEC: Spec>(interp: &mut Interpreter, host: &mut H) -> Return {
    pop_address!(interp, address);
    let (balance, is_cold) = host.balance(address);
    gas!(
        interp,
        if SPEC::enabled(ISTANBUL) {
            // EIP-1884: Repricing for trie-size-dependent opcodes
            gas::account_access_gas::<SPEC>(is_cold)
        } else if SPEC::enabled(TANGERINE) {
            400
        } else {
            20
        }
    );
    push!(interp, balance);

    Return::Continue
}

pub fn selfbalance<H: Host, SPEC: Spec>(interp: &mut Interpreter, host: &mut H) -> Return {
    // gas!(interp, gas::LOW);
    // EIP-1884: Repricing for trie-size-dependent opcodes
    check!(SPEC::enabled(ISTANBUL));

    let (balance, _) = host.balance(interp.contract.address);
    push!(interp, balance);

    Return::Continue
}

pub fn extcodesize<H: Host, SPEC: Spec>(interp: &mut Interpreter, host: &mut H) -> Return {
    pop_address!(interp, address);

    let (code, is_cold) = host.code(address);
    gas!(interp, gas::account_access_gas::<SPEC>(is_cold));

    push!(interp, U256::from(code.len()));

    Return::Continue
}

pub fn extcodehash<H: Host, SPEC: Spec>(interp: &mut Interpreter, host: &mut H) -> Return {
    check!(SPEC::enabled(CONSTANTINOPLE)); // EIP-1052: EXTCODEHASH opcode
    pop_address!(interp, address);
    let (code_hash, is_cold) = host.code_hash(address);
    gas!(
        interp,
        if SPEC::enabled(ISTANBUL) {
            // EIP-1884: Repricing for trie-size-dependent opcodes
            gas::account_access_gas::<SPEC>(is_cold)
        } else {
            400
        }
    );
    push_h256!(interp, code_hash);

    Return::Continue
}

pub fn extcodecopy<H: Host, SPEC: Spec>(interp: &mut Interpreter, host: &mut H) -> Return {
    pop_address!(interp, address);
    pop!(interp, memory_offset, code_offset, len_u256);

    let (code, is_cold) = host.code(address);
    gas_or_fail!(interp, gas::extcodecopy_cost::<SPEC>(len_u256, is_cold));
    let len = as_usize_or_fail!(len_u256, Return::OutOfGas);
    if len == 0 {
        return Return::Continue;
    }
    let memory_offset = as_usize_or_fail!(memory_offset, Return::OutOfGas);
    let code_offset = min(as_usize_saturated!(code_offset), code.len());
    memory_resize!(interp, memory_offset, len);

    // Safety: set_data is unsafe function and memory_resize ensures us that it is safe to call it
    interp
        .memory
        .set_data(memory_offset, code_offset, len, &code);
    Return::Continue
}

pub fn blockhash<H: Host>(interp: &mut Interpreter, host: &mut H) -> Return {
    // gas!(interp, gas::BLOCKHASH);

    pop!(interp, number);
    push_h256!(interp, host.block_hash(number));

    Return::Continue
}

pub fn sload<H: Host, SPEC: Spec>(interp: &mut Interpreter, host: &mut H) -> Return {
    pop!(interp, index);
    let (value, is_cold) = host.sload(interp.contract.address, index);
    gas!(interp, gas::sload_cost::<SPEC>(is_cold));
    push!(interp, value);
    Return::Continue
}

pub fn sstore<H: Host, SPEC: Spec>(interp: &mut Interpreter, host: &mut H) -> Return {
    check!(!SPEC::IS_STATIC_CALL);

    pop!(interp, index, value);
    let (original, old, new, is_cold) = host.sstore(interp.contract.address, index, value);
    gas_or_fail!(interp, {
        let remaining_gas = interp.gas.remaining();
        gas::sstore_cost::<SPEC>(original, old, new, remaining_gas, is_cold)
    });
    refund!(interp, gas::sstore_refund::<SPEC>(original, old, new));
    Return::Continue
}

pub fn log<H: Host, SPEC: Spec>(interp: &mut Interpreter, n: u8, host: &mut H) -> Return {
    check!(!SPEC::IS_STATIC_CALL);

    pop!(interp, offset, len);
    gas_or_fail!(interp, gas::log_cost(n, len));
    let len = as_usize_or_fail!(len, Return::OutOfGas);
    let data = if len == 0 {
        Bytes::new()
    } else {
        let offset = as_usize_or_fail!(offset, Return::OutOfGas);
        memory_resize!(interp, offset, len);
        Bytes::copy_from_slice(interp.memory.get_slice(offset, len))
    };
    let n = n as usize;
    if interp.stack.len() < n {
        return Return::StackUnderflow;
    }

    let mut topics = Vec::with_capacity(n);
    for _ in 0..(n) {
        let mut t = H256::zero();
        // Sefety: stack bounds already checked few lines above
        unsafe { interp.stack.pop_unsafe().to_big_endian(t.as_bytes_mut()) };
        topics.push(t);
    }

    host.log(interp.contract.address, topics, data);
    Return::Continue
}

pub fn selfdestruct<H: Host, SPEC: Spec>(interp: &mut Interpreter, host: &mut H) -> Return {
    check!(!SPEC::IS_STATIC_CALL);
    pop_address!(interp, target);

    let res = host.selfdestruct(interp.contract.address, target);

    // EIP-3529: Reduction in refunds
    if !SPEC::enabled(LONDON) && !res.previously_destroyed {
        refund!(interp, gas::SELFDESTRUCT)
    }
    gas!(interp, gas::selfdestruct_cost::<SPEC>(res));

    Return::SelfDestruct
}

fn gas_call_l64_after<SPEC: Spec>(interp: &mut Interpreter) -> Result<u64, Return> {
    if SPEC::enabled(TANGERINE) {
        //EIP-150: Gas cost changes for IO-heavy operations
        let gas = interp.gas().remaining();
        Ok(gas - gas / 64)
    } else {
        Ok(interp.gas().remaining())
    }
}

pub fn create<H: Host, SPEC: Spec>(
    interp: &mut Interpreter,
    is_create2: bool,
    host: &mut H,
) -> Return {
    check!(!SPEC::IS_STATIC_CALL);
    if is_create2 {
        check!(SPEC::enabled(CONSTANTINOPLE)); // EIP-1014: Skinny CREATE2
    }

    interp.return_data_buffer = Bytes::new();

    pop!(interp, value, code_offset, len);
    let len = as_usize_or_fail!(len, Return::OutOfGas);

    let code = if len == 0 {
        Bytes::new()
    } else {
        let code_offset = as_usize_or_fail!(code_offset, Return::OutOfGas);
        memory_resize!(interp, code_offset, len);
        Bytes::copy_from_slice(interp.memory.get_slice(code_offset, len))
    };

    let scheme = if is_create2 {
        pop!(interp, salt);
        gas_or_fail!(interp, gas::create2_cost(len));
        CreateScheme::Create2 { salt }
    } else {
        gas!(interp, gas::CREATE);
        CreateScheme::Create
    };

    // take remaining gas and deduce l64 part of it.
    let gas_limit = try_or_fail!(gas_call_l64_after::<SPEC>(interp));
    gas!(interp, gas_limit);

    let mut create_input = CreateInputs {
        caller: interp.contract.address,
        scheme,
        value,
        init_code: code,
        gas_limit,
    };

    let (reason, address, gas, return_data) = host.create::<SPEC>(&mut create_input);
    interp.return_data_buffer = return_data;
    let created_address: H256 = if matches!(reason, return_ok!()) {
        address.map(|a| a.into()).unwrap_or_default()
    } else {
        H256::default()
    };
    push_h256!(interp, created_address);
    // reimburse gas that is not spend
    interp.gas.reimburse_unspend(&reason, gas);
    match reason {
        Return::FatalNotSupported => Return::FatalNotSupported,
        _ => interp.add_next_gas_block(interp.program_counter() - 1),
    }
}

pub fn call<H: Host, SPEC: Spec>(
    interp: &mut Interpreter,
    scheme: CallScheme,
    host: &mut H,
) -> Return {
    match scheme {
        CallScheme::DelegateCall => check!(SPEC::enabled(HOMESTEAD)), // EIP-7: DELEGATECALL
        CallScheme::StaticCall => check!(SPEC::enabled(BYZANTINE)), // EIP-214: New opcode STATICCALL
        _ => (),
    }
    interp.return_data_buffer = Bytes::new();

    pop!(interp, local_gas_limit);
    pop_address!(interp, to);
    let local_gas_limit = if local_gas_limit > U256::from(u64::MAX) {
        u64::MAX
    } else {
        local_gas_limit.as_u64()
    };

    let value = match scheme {
        CallScheme::CallCode => {
            pop!(interp, value);
            value
        }
        CallScheme::Call => {
            pop!(interp, value);
            if SPEC::IS_STATIC_CALL && !value.is_zero() {
                return Return::CallNotAllowedInsideStatic;
            }
            value
        }
        CallScheme::DelegateCall | CallScheme::StaticCall => U256::zero(),
    };

    pop!(interp, in_offset, in_len, out_offset, out_len);

    let in_len = as_usize_or_fail!(in_len, Return::OutOfGas);
    let input = if in_len != 0 {
        let in_offset = as_usize_or_fail!(in_offset, Return::OutOfGas);
        memory_resize!(interp, in_offset, in_len);
        Bytes::copy_from_slice(interp.memory.get_slice(in_offset, in_len))
    } else {
        Bytes::new()
    };

    let out_len = as_usize_or_fail!(out_len, Return::OutOfGas);
    let out_offset = if out_len != 0 {
        let out_offset = as_usize_or_fail!(out_offset, Return::OutOfGas);
        memory_resize!(interp, out_offset, out_len);
        out_offset
    } else {
        usize::MAX //unrealistic value so we are sure it is not used
    };

    let context = match scheme {
        CallScheme::Call | CallScheme::StaticCall => CallContext {
            address: to,
            caller: interp.contract.address,
            code_address: to,
            apparent_value: value,
            scheme,
        },
        CallScheme::CallCode => CallContext {
            address: interp.contract.address,
            caller: interp.contract.address,
            code_address: to,
            apparent_value: value,
            scheme,
        },
        CallScheme::DelegateCall => CallContext {
            address: interp.contract.address,
            caller: interp.contract.caller,
            code_address: to,
            apparent_value: interp.contract.value,
            scheme,
        },
    };

    let transfer = if scheme == CallScheme::Call {
        Transfer {
            source: interp.contract.address,
            target: to,
            value,
        }
    } else if scheme == CallScheme::CallCode {
        Transfer {
            source: interp.contract.address,
            target: interp.contract.address,
            value,
        }
    } else {
        //this is dummy send for StaticCall and DelegateCall, it should do nothing and dont touch anything.
        Transfer {
            source: interp.contract.address,
            target: interp.contract.address,
            value: U256::zero(),
        }
    };

    // load account and calculate gas cost.
    let (is_cold, exist) = host.load_account(to);
    let is_new = !exist;
    //let is_cold = false;
    gas!(
        interp,
        gas::call_cost::<SPEC>(
            value,
            is_new,
            is_cold,
            matches!(scheme, CallScheme::Call | CallScheme::CallCode),
            matches!(scheme, CallScheme::Call | CallScheme::StaticCall),
        )
    );

    // take l64 part of gas_limit
    let global_gas_limit = try_or_fail!(gas_call_l64_after::<SPEC>(interp));
    let mut gas_limit = min(global_gas_limit, local_gas_limit);

    gas!(interp, gas_limit);

    // add call stipend if there is value to be transfered.
    if matches!(scheme, CallScheme::Call | CallScheme::CallCode) && !transfer.value.is_zero() {
        gas_limit = gas_limit.saturating_add(gas::CALL_STIPEND);
    }
    let is_static = matches!(scheme, CallScheme::StaticCall);

    let mut call_input = CallInputs {
        contract: to,
        transfer,
        input,
        gas_limit,
        context,
    };
    // CALL CONTRACT, with static or ordinary spec.
    let (reason, gas, return_data) = if is_static {
        host.call::<SPEC::STATIC>(&mut call_input)
    } else {
        host.call::<SPEC>(&mut call_input)
    };
    interp.return_data_buffer = return_data;

    let target_len = min(out_len, interp.return_data_buffer.len());
    // return unspend gas.
    interp.gas.reimburse_unspend(&reason, gas);
    match reason {
        return_ok!() => {
            interp
                .memory
                .set(out_offset, &interp.return_data_buffer[..target_len]);
            push!(interp, U256::one());
        }
        return_revert!() => {
            push!(interp, U256::zero());
            interp
                .memory
                .set(out_offset, &interp.return_data_buffer[..target_len]);
        }
        _ => {
            push!(interp, U256::zero());
        }
    }
    interp.add_next_gas_block(interp.program_counter() - 1)
}
