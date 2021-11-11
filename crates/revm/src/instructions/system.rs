use super::gas;
use crate::{
    machine::Machine, CallContext, CallScheme, CreateScheme, Handler, Return, Spec, Transfer,
};
use crate::{return_ok, return_revert};
// 	CallScheme, Capture, CallContext, CreateScheme, ,
// 	, Runtime, Transfer,
// };
use crate::{alloc::vec::Vec, spec::SpecId::*};
use bytes::Bytes;
use core::cmp::min;
use primitive_types::{H256, U256};
use sha3::{Digest, Keccak256};

pub fn sha3(machine: &mut Machine) -> Return {
    pop_u256!(machine, from, len);
    gas_or_fail!(machine, gas::sha3_cost(len));

    memory_resize!(machine, from, len);
    let data = if len.is_zero() {
        Bytes::new()
    } else {
        let from = as_usize_or_fail!(from);
        let len = as_usize_or_fail!(len);

        machine.memory.get(from, len)
    };

    let ret = Keccak256::digest(data.as_ref());
    push!(machine, H256::from_slice(ret.as_slice()));

    Return::Continue
}

pub fn chainid<H: Handler, SPEC: Spec>(machine: &mut Machine, handler: &mut H) -> Return {
    check!(SPEC::enabled(ISTANBUL)); // EIP-1344: ChainID opcode
    gas!(machine, gas::BASE);

    push_u256!(machine, handler.env().cfg.chain_id);

    Return::Continue
}

pub fn address(machine: &mut Machine) -> Return {
    gas!(machine, gas::BASE);

    let ret = H256::from(machine.contract.address);
    push!(machine, ret);

    Return::Continue
}

pub fn balance<H: Handler, SPEC: Spec>(machine: &mut Machine, handler: &mut H) -> Return {
    pop!(machine, address);
    let (balance, is_cold) = handler.balance(address.into());
    gas!(
        machine,
        if SPEC::enabled(ISTANBUL) {
            // EIP-1884: Repricing for trie-size-dependent opcodes
            gas::account_access_gas::<SPEC>(is_cold)
        } else if SPEC::enabled(TANGERINE) {
            400
        } else {
            20
        }
    );
    push_u256!(machine, balance);

    Return::Continue
}

pub fn selfbalance<H: Handler, SPEC: Spec>(machine: &mut Machine, handler: &mut H) -> Return {
    check!(SPEC::enabled(ISTANBUL)); // EIP-1884: Repricing for trie-size-dependent opcodes
    let (balance, _) = handler.balance(machine.contract.address);
    gas!(machine, gas::LOW);
    push_u256!(machine, balance);

    Return::Continue
}

pub fn basefee<H: Handler, SPEC: Spec>(machine: &mut Machine, handler: &mut H) -> Return {
    check!(SPEC::enabled(LONDON)); // EIP-3198: BASEFEE opcode
    let basefee = handler.env().block.basefee;
    gas!(machine, gas::BASE);
    push_u256!(machine, basefee);

    Return::Continue
}

pub fn origin<H: Handler>(machine: &mut Machine, handler: &mut H) -> Return {
    gas!(machine, gas::BASE);

    let ret = H256::from(handler.env().tx.caller);
    push!(machine, ret);

    Return::Continue
}

pub fn caller(machine: &mut Machine) -> Return {
    gas!(machine, gas::BASE);

    let ret = H256::from(machine.contract.caller);
    push!(machine, ret);

    Return::Continue
}

pub fn callvalue(machine: &mut Machine) -> Return {
    gas!(machine, gas::BASE);

    let mut ret = H256::default();
    machine.contract.value.to_big_endian(&mut ret[..]);
    push!(machine, ret);

    Return::Continue
}

pub fn gasprice<H: Handler>(machine: &mut Machine, handler: &mut H) -> Return {
    gas!(machine, gas::BASE);

    let mut ret = H256::default();
    handler
        .env()
        .effective_gas_price()
        .to_big_endian(&mut ret[..]);
    push!(machine, ret);

    Return::Continue
}

pub fn extcodesize<H: Handler, SPEC: Spec>(machine: &mut Machine, handler: &mut H) -> Return {
    pop!(machine, address);

    let (code, is_cold) = handler.code(address.into());
    gas!(machine, gas::account_access_gas::<SPEC>(is_cold));

    push_u256!(machine, U256::from(code.len()));

    Return::Continue
}

pub fn extcodehash<H: Handler, SPEC: Spec>(machine: &mut Machine, handler: &mut H) -> Return {
    check!(SPEC::enabled(CONSTANTINOPLE)); // EIP-1052: EXTCODEHASH opcode
    pop!(machine, address);
    let (code_hash, is_cold) = handler.code_hash(address.into());
    gas!(
        machine,
        if SPEC::enabled(ISTANBUL) {
            // EIP-1884: Repricing for trie-size-dependent opcodes
            gas::account_access_gas::<SPEC>(is_cold)
        } else {
            400
        }
    );
    push!(machine, code_hash);

    Return::Continue
}

pub fn extcodecopy<H: Handler, SPEC: Spec>(machine: &mut Machine, handler: &mut H) -> Return {
    pop!(machine, address);
    pop_u256!(machine, memory_offset, code_offset, len);

    let (code, is_cold) = handler.code(address.into());
    gas_or_fail!(machine, gas::extcodecopy_cost::<SPEC>(len, is_cold));

    memory_resize!(machine, memory_offset, len);
    machine
        .memory
        .copy_large(memory_offset, code_offset, len, &code)
}

pub fn returndatasize<SPEC: Spec>(machine: &mut Machine) -> Return {
    check!(SPEC::enabled(BYZANTINE)); // EIP-211: New opcodes: RETURNDATASIZE and RETURNDATACOPY
    gas!(machine, gas::BASE);

    let size = U256::from(machine.return_data_buffer.len());
    push_u256!(machine, size);

    Return::Continue
}

pub fn returndatacopy<SPEC: Spec>(machine: &mut Machine) -> Return {
    check!(SPEC::enabled(BYZANTINE)); // EIP-211: New opcodes: RETURNDATASIZE and RETURNDATACOPY
    pop_u256!(machine, memory_offset, data_offset, len);
    gas_or_fail!(machine, gas::verylowcopy_cost(len));
    memory_resize!(machine, memory_offset, len);
    if data_offset
        .checked_add(len)
        .map(|l| l > U256::from(machine.return_data_buffer.len()))
        .unwrap_or(true)
    {
        return Return::OutOfOffset;
    }

    machine
        .memory
        .copy_large(memory_offset, data_offset, len, &machine.return_data_buffer)
}

pub fn blockhash<H: Handler>(machine: &mut Machine, handler: &mut H) -> Return {
    gas!(machine, gas::BLOCKHASH);

    pop_u256!(machine, number);
    push!(machine, handler.block_hash(number));

    Return::Continue
}

pub fn coinbase<H: Handler>(machine: &mut Machine, handler: &mut H) -> Return {
    gas!(machine, gas::BASE);

    push!(machine, handler.env().block.coinbase.into());
    Return::Continue
}

pub fn timestamp<H: Handler>(machine: &mut Machine, handler: &mut H) -> Return {
    gas!(machine, gas::BASE);
    push_u256!(machine, handler.env().block.timestamp);
    Return::Continue
}

pub fn number<H: Handler>(machine: &mut Machine, handler: &mut H) -> Return {
    gas!(machine, gas::BASE);

    push_u256!(machine, handler.env().block.number);
    Return::Continue
}

pub fn difficulty<H: Handler>(machine: &mut Machine, handler: &mut H) -> Return {
    gas!(machine, gas::BASE);

    push_u256!(machine, handler.env().block.difficulty);
    Return::Continue
}

pub fn gaslimit<H: Handler>(machine: &mut Machine, handler: &mut H) -> Return {
    gas!(machine, gas::BASE);

    push_u256!(machine, handler.env().block.gas_limit);
    Return::Continue
}

pub fn sload<H: Handler, SPEC: Spec>(machine: &mut Machine, handler: &mut H) -> Return {
    pop!(machine, index);
    let (value, is_cold) = handler.sload(machine.contract.address, index);
    inspect!(
        handler,
        sload,
        &machine.contract.address,
        &index,
        &value,
        is_cold
    );
    gas!(machine, gas::sload_cost::<SPEC>(is_cold));
    push!(machine, value);
    Return::Continue
}

pub fn sstore<H: Handler, SPEC: Spec>(machine: &mut Machine, handler: &mut H) -> Return {
    check!(!SPEC::IS_STATIC_CALL);

    pop!(machine, index, value);
    let (original, old, new, is_cold) = handler.sstore(machine.contract.address, index, value);
    inspect!(
        handler,
        sstore,
        machine.contract.address,
        index,
        new,
        old,
        original,
        is_cold
    );
    gas_or_fail!(machine, {
        let remaining_gas = machine.gas.remaining();
        gas::sstore_cost::<SPEC>(original, old, new, remaining_gas, is_cold)
    });
    refund!(machine, gas::sstore_refund::<SPEC>(original, old, new));
    Return::Continue
}

pub fn gas(machine: &mut Machine) -> Return {
    gas!(machine, gas::BASE);

    push_u256!(machine, U256::from(machine.gas.remaining()));
    Return::Continue
}

pub fn log<H: Handler, SPEC: Spec>(machine: &mut Machine, n: u8, handler: &mut H) -> Return {
    check!(!SPEC::IS_STATIC_CALL);

    pop_u256!(machine, offset, len);
    gas_or_fail!(machine, gas::log_cost(n, len));
    memory_resize!(machine, offset, len);
    let data = if len.is_zero() {
        Bytes::new()
    } else {
        let offset = as_usize_or_fail!(offset);
        let len = as_usize_or_fail!(len);

        machine.memory.get(offset, len)
    };
    let n = n as usize;
    if machine.stack.len() < n {
        return Return::StackUnderflow;
    }

    let mut topics = Vec::with_capacity(n);
    for _ in 0..(n) {
        /*** SAFETY stack bounds already checked few lines above */
        topics.push(unsafe { machine.stack.pop_unsafe() });
    }

    handler.log(machine.contract.address, topics, data);
    Return::Continue
}

pub fn selfdestruct<H: Handler, SPEC: Spec>(machine: &mut Machine, handler: &mut H) -> Return {
    check!(!SPEC::IS_STATIC_CALL);
    pop!(machine, target);

    let res = handler.selfdestruct(machine.contract.address, target.into());
    inspect!(handler, selfdestruct);

    // EIP-3529: Reduction in refunds
    if !SPEC::enabled(LONDON) && !res.previously_destroyed {
        refund!(machine, gas::SELFDESTRUCT)
    }
    gas!(machine, gas::selfdestruct_cost::<SPEC>(res));

    Return::SelfDestruct
}

#[inline(always)]
fn gas_call_l64_after<SPEC: Spec>(machine: &mut Machine) -> Result<u64, Return> {
    if SPEC::enabled(TANGERINE) {
        //EIP-150: Gas cost changes for IO-heavy operations
        let gas = machine.gas().remaining();
        Ok(gas - gas / 64)
    } else {
        Ok(machine.gas().remaining())
    }
}

pub fn create<H: Handler, SPEC: Spec>(
    machine: &mut Machine,
    is_create2: bool,
    handler: &mut H,
) -> Return {
    check!(!SPEC::IS_STATIC_CALL);
    if is_create2 {
        check!(SPEC::enabled(CONSTANTINOPLE)); // EIP-1014: Skinny CREATE2
    }

    machine.return_data_buffer = Bytes::new();

    pop_u256!(machine, value, code_offset, len);

    memory_resize!(machine, code_offset, len);
    let code = if len.is_zero() {
        Bytes::new()
    } else {
        let code_offset = as_usize_or_fail!(code_offset);
        let len = as_usize_or_fail!(len);

        machine.memory.get(code_offset, len)
    };
    let scheme = if is_create2 {
        pop!(machine, salt);
        gas_or_fail!(machine, gas::create2_cost(len));
        CreateScheme::Create2 { salt }
    } else {
        gas!(machine, gas::CREATE);
        CreateScheme::Create
    };

    // take remaining gas and deduce l64 part of it.
    let gas_limit = try_or_fail!(gas_call_l64_after::<SPEC>(machine));
    gas!(machine, gas_limit);

    inspect!(
        handler,
        create,
        machine.contract.address,
        &scheme,
        value,
        &code,
        gas_limit
    );

    let (reason, address, gas, return_data) =
        handler.create::<SPEC>(machine.contract.address, scheme, value, code, gas_limit);
    machine.return_data_buffer = return_data;
    let created_address: H256 = if matches!(reason, return_ok!()) {
        address.map(|a| a.into()).unwrap_or_default()
    } else {
        H256::default()
    };
    inspect!(handler, create_return, created_address);
    push!(machine, created_address);
    // reimburse gas that is not spend
    machine.gas.reimburse_unspend(&reason, gas);
    match reason {
        Return::FatalNotSupported => Return::FatalNotSupported,
        _ => Return::Continue,
    }
}

pub fn call<H: Handler, SPEC: Spec>(
    machine: &mut Machine,
    scheme: CallScheme,
    handler: &mut H,
) -> Return {
    match scheme {
        CallScheme::DelegateCall => check!(SPEC::enabled(HOMESTEAD)), // EIP-7: DELEGATECALL
        CallScheme::StaticCall => check!(SPEC::enabled(BYZANTINE)), // EIP-214: New opcode STATICCALL
        _ => (),
    }
    machine.return_data_buffer = Bytes::new();

    pop_u256!(machine, local_gas_limit);
    pop!(machine, to);
    let local_gas_limit = if local_gas_limit > U256::from(u64::MAX) {
        u64::MAX
    } else {
        local_gas_limit.as_u64()
    };

    let value = match scheme {
        CallScheme::CallCode => {
            pop_u256!(machine, value);
            value
        }
        CallScheme::Call => {
            pop_u256!(machine, value);
            if SPEC::IS_STATIC_CALL && !value.is_zero() {
                return Return::CallNotAllowedInsideStatic;
            }
            value
        }
        CallScheme::DelegateCall | CallScheme::StaticCall => U256::zero(),
    };

    pop_u256!(machine, in_offset, in_len, out_offset, out_len);

    memory_resize!(machine, in_offset, in_len);
    memory_resize!(machine, out_offset, out_len);

    let input = if in_len.is_zero() {
        Bytes::new()
    } else {
        let in_offset = as_usize_or_fail!(in_offset);
        let in_len = as_usize_or_fail!(in_len);

        machine.memory.get(in_offset, in_len)
    };

    let context = match scheme {
        CallScheme::Call | CallScheme::StaticCall => CallContext {
            address: to.into(),
            caller: machine.contract.address,
            apparent_value: value,
        },
        CallScheme::CallCode => CallContext {
            address: machine.contract.address,
            caller: machine.contract.address,
            apparent_value: value,
        },
        CallScheme::DelegateCall => CallContext {
            address: machine.contract.address,
            caller: machine.contract.caller,
            apparent_value: machine.contract.value,
        },
    };

    let transfer = if scheme == CallScheme::Call {
        Transfer {
            source: machine.contract.address,
            target: to.into(),
            value,
        }
    } else if scheme == CallScheme::CallCode {
        Transfer {
            source: machine.contract.address,
            target: machine.contract.address,
            value,
        }
    } else {
        //this is dummy send for StaticCall and DelegateCall, it should do nothing and dont touch anything.
        Transfer {
            source: machine.contract.address,
            target: machine.contract.address,
            value: U256::zero(),
        }
    };

    let to = to.into();
    // load account and calculate gas cost.
    let (is_cold, exist) = handler.load_account(to);
    let is_new = !exist;
    //let is_cold = false;
    gas!(
        machine,
        gas::call_cost::<SPEC>(
            value,
            is_new,
            is_cold,
            matches!(scheme, CallScheme::Call | CallScheme::CallCode),
            matches!(scheme, CallScheme::Call | CallScheme::StaticCall),
        )
    );

    // take l64 part of gas_limit
    let global_gas_limit = try_or_fail!(gas_call_l64_after::<SPEC>(machine));
    let mut gas_limit = min(global_gas_limit, local_gas_limit);

    gas!(machine, gas_limit);

    // add call stipend if there is value to be transfered.
    if matches!(scheme, CallScheme::Call | CallScheme::CallCode) && !transfer.value.is_zero() {
        gas_limit = gas_limit.saturating_add(gas::CALL_STIPEND);
    }
    let is_static = matches!(scheme, CallScheme::StaticCall);
    inspect!(handler, call, to, &context, &transfer, &input, gas_limit, is_static);

    // CALL CONTRACT, with static or ordinary spec.
    let (reason, gas, return_data) = if is_static {
        handler.call::<SPEC::STATIC>(to, transfer, input, gas_limit, context)
    } else {
        handler.call::<SPEC>(to, transfer, input, gas_limit, context)
    };
    machine.return_data_buffer = return_data;

    let target_len = min(out_len, U256::from(machine.return_data_buffer.len()));
    // return unspend gas.
    machine.gas.reimburse_unspend(&reason, gas);
    match reason {
        return_ok!() => {
            if machine.memory.copy_large(
                out_offset,
                U256::zero(),
                target_len,
                &machine.return_data_buffer,
            ) == Return::Continue
            {
                push_u256!(machine, U256::one());
                Return::Continue
            } else {
                push_u256!(machine, U256::zero());
                Return::Continue
            }
        }
        return_revert!() => {
            push_u256!(machine, U256::zero());
            let _ = machine.memory.copy_large(
                out_offset,
                U256::zero(),
                target_len,
                &machine.return_data_buffer,
            );
            Return::Continue
        }
        _ => {
            push_u256!(machine, U256::zero());
            Return::Continue
        }
    }
}
