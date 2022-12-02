use crate::{
    alloc::vec::Vec,
    bits::{B160, B256},
    gas::{self, COLD_ACCOUNT_ACCESS_COST, WARM_STORAGE_READ_COST},
    interpreter::Interpreter,
    return_ok, return_revert, CallContext, CallInputs, CallScheme, CreateInputs, CreateScheme,
    Host, Return, Spec,
    SpecId::*,
    Transfer, U256,
};
use bytes::Bytes;
use core::cmp::min;

pub fn balance<H: Host, SPEC: Spec>(interp: &mut Interpreter, host: &mut H) -> Return {
    pop_address!(interp, address);
    let ret = host.balance(address);
    if ret.is_none() {
        return Return::FatalExternalError;
    }
    let (balance, is_cold) = ret.unwrap();
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
    let ret = host.balance(interp.contract.address);
    if ret.is_none() {
        return Return::FatalExternalError;
    }
    let (balance, _) = ret.unwrap();
    push!(interp, balance);

    Return::Continue
}

pub fn extcodesize<H: Host, SPEC: Spec>(interp: &mut Interpreter, host: &mut H) -> Return {
    pop_address!(interp, address);
    let ret = host.code(address);
    if ret.is_none() {
        return Return::FatalExternalError;
    }
    let (code, is_cold) = ret.unwrap();
    if SPEC::enabled(BERLIN) && is_cold {
        // WARM_STORAGE_READ_COST is already calculated in gas block
        gas!(interp, COLD_ACCOUNT_ACCESS_COST - WARM_STORAGE_READ_COST);
    }

    push!(interp, U256::from(code.len()));

    Return::Continue
}

pub fn extcodehash<H: Host, SPEC: Spec>(interp: &mut Interpreter, host: &mut H) -> Return {
    check!(SPEC::enabled(CONSTANTINOPLE)); // EIP-1052: EXTCODEHASH opcode
    pop_address!(interp, address);
    let ret = host.code_hash(address);
    if ret.is_none() {
        return Return::FatalExternalError;
    }
    let (code_hash, is_cold) = ret.unwrap();
    if SPEC::enabled(BERLIN) && is_cold {
        // WARM_STORAGE_READ_COST is already calculated in gas block
        gas!(interp, COLD_ACCOUNT_ACCESS_COST - WARM_STORAGE_READ_COST);
    }
    push_b256!(interp, code_hash);

    Return::Continue
}

pub fn extcodecopy<H: Host, SPEC: Spec>(interp: &mut Interpreter, host: &mut H) -> Return {
    pop_address!(interp, address);
    pop!(interp, memory_offset, code_offset, len_u256);

    let ret = host.code(address);
    if ret.is_none() {
        return Return::FatalExternalError;
    }
    let (code, is_cold) = ret.unwrap();

    let len = as_usize_or_fail!(len_u256, Return::OutOfGas);
    gas_or_fail!(interp, gas::extcodecopy_cost::<SPEC>(len as u64, is_cold));
    if len == 0 {
        return Return::Continue;
    }
    let memory_offset = as_usize_or_fail!(memory_offset, Return::OutOfGas);
    let code_offset = min(as_usize_saturated!(code_offset), code.len());
    memory_resize!(interp, memory_offset, len);

    // Safety: set_data is unsafe function and memory_resize ensures us that it is safe to call it
    interp
        .memory
        .set_data(memory_offset, code_offset, len, code.bytes());
    Return::Continue
}

pub fn blockhash<H: Host>(interp: &mut Interpreter, host: &mut H) -> Return {
    // gas!(interp, gas::BLOCKHASH);
    pop_top!(interp, number);

    if let Some(diff) = host.env().block.number.checked_sub(*number) {
        let diff = as_usize_saturated!(diff);
        // blockhash should push zero if number is same as current block number.
        if diff <= 256 && diff != 0 {
            let ret = host.block_hash(*number);
            if ret.is_none() {
                return Return::FatalExternalError;
            }
            *number = U256::from_be_bytes(*ret.unwrap());
            return Return::Continue;
        }
    }
    *number = U256::ZERO;
    Return::Continue
}

pub fn sload<H: Host, SPEC: Spec>(interp: &mut Interpreter, host: &mut H) -> Return {
    pop!(interp, index);

    let ret = host.sload(interp.contract.address, index);
    if ret.is_none() {
        return Return::FatalExternalError;
    }
    let (value, is_cold) = ret.unwrap();
    gas!(interp, gas::sload_cost::<SPEC>(is_cold));
    push!(interp, value);
    Return::Continue
}

pub fn sstore<H: Host, SPEC: Spec>(interp: &mut Interpreter, host: &mut H) -> Return {
    check!(!SPEC::IS_STATIC_CALL);

    pop!(interp, index, value);
    let ret = host.sstore(interp.contract.address, index, value);
    if ret.is_none() {
        return Return::FatalExternalError;
    }
    let (original, old, new, is_cold) = ret.unwrap();
    gas_or_fail!(interp, {
        let remaining_gas = interp.gas.remaining();
        gas::sstore_cost::<SPEC>(original, old, new, remaining_gas, is_cold)
    });
    refund!(interp, gas::sstore_refund::<SPEC>(original, old, new));
    interp.add_next_gas_block(interp.program_counter() - 1)
}

pub fn log<H: Host, SPEC: Spec>(interp: &mut Interpreter, n: u8, host: &mut H) -> Return {
    check!(!SPEC::IS_STATIC_CALL);

    pop!(interp, offset, len);
    let len = as_usize_or_fail!(len, Return::OutOfGas);
    gas_or_fail!(interp, gas::log_cost(n, len as u64));
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
        // Safety: stack bounds already checked few lines above
        topics.push(B256(unsafe { interp.stack.pop_unsafe().to_be_bytes() }));
    }

    host.log(interp.contract.address, topics, data);
    Return::Continue
}

pub fn selfdestruct<H: Host, SPEC: Spec>(interp: &mut Interpreter, host: &mut H) -> Return {
    check!(!SPEC::IS_STATIC_CALL);
    pop_address!(interp, target);

    let res = host.selfdestruct(interp.contract.address, target);
    if res.is_none() {
        return Return::FatalExternalError;
    }
    let res = res.unwrap();

    // EIP-3529: Reduction in refunds
    if !SPEC::enabled(LONDON) && !res.previously_destroyed {
        refund!(interp, gas::SELFDESTRUCT)
    }
    gas!(interp, gas::selfdestruct_cost::<SPEC>(res));

    Return::SelfDestruct
}

pub fn create<H: Host, SPEC: Spec>(
    interp: &mut Interpreter,
    is_create2: bool,
    host: &mut H,
) -> Return {
    check!(!SPEC::IS_STATIC_CALL);
    if is_create2 {
        // EIP-1014: Skinny CREATE2
        check!(SPEC::enabled(PETERSBURG));
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

    let mut gas_limit = interp.gas().remaining();

    // EIP-150: Gas cost changes for IO-heavy operations
    if SPEC::enabled(TANGERINE) {
        // take remaining gas and deduce l64 part of it.
        gas_limit -= gas_limit / 64
    }
    gas!(interp, gas_limit);

    let mut create_input = CreateInputs {
        caller: interp.contract.address,
        scheme,
        value,
        init_code: code,
        gas_limit,
    };

    let (return_reason, address, gas, return_data) = host.create::<SPEC>(&mut create_input);
    interp.return_data_buffer = return_data;

    match return_reason {
        return_ok!() => {
            push_b256!(interp, address.unwrap_or_default().into());
            interp.gas.erase_cost(gas.remaining());
            interp.gas.record_refund(gas.refunded());
        }
        return_revert!() => {
            push_b256!(interp, B256::zero());
            interp.gas.erase_cost(gas.remaining());
        }
        Return::FatalExternalError => return Return::FatalExternalError,
        _ => {
            push_b256!(interp, B256::zero());
        }
    }
    interp.add_next_gas_block(interp.program_counter() - 1)
}

pub fn call<H: Host, SPEC: Spec>(
    interp: &mut Interpreter,
    scheme: CallScheme,
    host: &mut H,
) -> Return {
    match scheme {
        CallScheme::DelegateCall => check!(SPEC::enabled(HOMESTEAD)), // EIP-7: DELEGATECALL
        CallScheme::StaticCall => check!(SPEC::enabled(BYZANTIUM)), // EIP-214: New opcode STATICCALL
        _ => (),
    }
    interp.return_data_buffer = Bytes::new();

    pop!(interp, local_gas_limit);
    pop_address!(interp, to);
    let local_gas_limit = u64::try_from(local_gas_limit).unwrap_or(u64::MAX);

    let value = match scheme {
        CallScheme::CallCode => {
            pop!(interp, value);
            value
        }
        CallScheme::Call => {
            pop!(interp, value);
            if SPEC::IS_STATIC_CALL && value != U256::ZERO {
                return Return::CallNotAllowedInsideStatic;
            }
            value
        }
        CallScheme::DelegateCall | CallScheme::StaticCall => U256::ZERO,
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
            value: U256::ZERO,
        }
    };

    // load account and calculate gas cost.
    let res = host.load_account(to);
    if res.is_none() {
        return Return::FatalExternalError;
    }
    let (is_cold, exist) = res.unwrap();
    let is_new = !exist;

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
    let mut gas_limit = if SPEC::enabled(TANGERINE) {
        //EIP-150: Gas cost changes for IO-heavy operations
        let gas = interp.gas().remaining();
        min(gas - gas / 64, local_gas_limit)
    } else {
        local_gas_limit
    };

    gas!(interp, gas_limit);

    // add call stipend if there is value to be transferred.
    if matches!(scheme, CallScheme::Call | CallScheme::CallCode) && transfer.value != U256::ZERO {
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

    match reason {
        return_ok!() => {
            // return unspend gas.
            interp.gas.erase_cost(gas.remaining());
            interp.gas.record_refund(gas.refunded());
            interp
                .memory
                .set(out_offset, &interp.return_data_buffer[..target_len]);
            push!(interp, U256::from(1));
        }
        return_revert!() => {
            interp.gas.erase_cost(gas.remaining());
            interp
                .memory
                .set(out_offset, &interp.return_data_buffer[..target_len]);
            push!(interp, U256::ZERO);
        }
        Return::FatalExternalError => return Return::FatalExternalError,
        _ => {
            push!(interp, U256::ZERO);
        }
    }
    interp.add_next_gas_block(interp.program_counter() - 1)
}
