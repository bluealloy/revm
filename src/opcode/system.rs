use super::{gas, Control};
use crate::{
    error::{ExitError, ExitFatal, ExitReason, ExitSucceed},
    machine::Machine,
    CallContext, CallScheme, CreateScheme, Handler, Spec, Transfer,
};
// 	CallScheme, Capture, CallContext, CreateScheme, ,
// 	, Runtime, Transfer,
// };
use crate::{collection::vec::Vec, spec::SpecId::*};
use bytes::Bytes;
use core::cmp::min;
use primitive_types::{H256, U256};
use sha3::{Digest, Keccak256};

pub fn sha3(machine: &mut Machine) -> Control {
    pop_u256!(machine, from, len);
    gas_or_fail!(machine, gas::sha3_cost(len));

    memory_resize!(machine, from, len);
    let data = if len == U256::zero() {
        Bytes::new()
    } else {
        let from = as_usize_or_fail!(from);
        let len = as_usize_or_fail!(len);

        machine.memory_mut().get(from, len)
    };

    let ret = Keccak256::digest(data.as_ref());
    push!(machine, H256::from_slice(ret.as_slice()));

    Control::Continue
}

pub fn chainid<H: Handler, SPEC: Spec>(machine: &mut Machine, handler: &mut H) -> Control {
    check!(SPEC::enabled(ISTANBUL)); // EIP-1344: ChainID opcode
    gas!(machine, gas::BASE);

    push_u256!(machine, handler.env().chain_id);

    Control::Continue
}

pub fn address(machine: &mut Machine) -> Control {
    gas!(machine, gas::BASE);

    let ret = H256::from(machine.contract.address);
    push!(machine, ret);

    Control::Continue
}

pub fn balance<H: Handler, SPEC: Spec>(machine: &mut Machine, handler: &mut H) -> Control {
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

    Control::Continue
}

pub fn selfbalance<H: Handler, SPEC: Spec>(machine: &mut Machine, handler: &mut H) -> Control {
    check!(SPEC::enabled(ISTANBUL)); // EIP-1884: Repricing for trie-size-dependent opcodes
    let (balance, _) = handler.balance(machine.contract.address);
    gas!(machine, gas::LOW);
    push_u256!(machine, balance);

    Control::Continue
}

pub fn origin<H: Handler>(machine: &mut Machine, handler: &mut H) -> Control {
    gas!(machine, gas::BASE);

    let ret = H256::from(handler.env().origin);
    push!(machine, ret);

    Control::Continue
}

pub fn caller(machine: &mut Machine) -> Control {
    gas!(machine, gas::BASE);

    let ret = H256::from(machine.contract.caller);
    push!(machine, ret);

    Control::Continue
}

pub fn callvalue(machine: &mut Machine) -> Control {
    gas!(machine, gas::BASE);

    let mut ret = H256::default();
    machine.contract.value.to_big_endian(&mut ret[..]);
    push!(machine, ret);

    Control::Continue
}

pub fn gasprice<H: Handler>(machine: &mut Machine, handler: &mut H) -> Control {
    gas!(machine, gas::BASE);

    let mut ret = H256::default();
    handler.env().gas_price.to_big_endian(&mut ret[..]);
    push!(machine, ret);

    Control::Continue
}

pub fn extcodesize<H: Handler, SPEC: Spec>(machine: &mut Machine, handler: &mut H) -> Control {
    pop!(machine, address);

    let (code, is_cold) = handler.code(address.into());
    gas!(machine, gas::account_access_gas::<SPEC>(is_cold));

    push_u256!(machine, U256::from(code.len()));

    Control::Continue
}

pub fn extcodehash<H: Handler, SPEC: Spec>(machine: &mut Machine, handler: &mut H) -> Control {
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

    Control::Continue
}

pub fn extcodecopy<H: Handler, SPEC: Spec>(machine: &mut Machine, handler: &mut H) -> Control {
    pop!(machine, address);
    pop_u256!(machine, memory_offset, code_offset, len);

    let (code, is_cold) = handler.code(address.into());
    gas_or_fail!(machine, gas::extcodecopy_cost::<SPEC>(len, is_cold));

    memory_resize!(machine, memory_offset, len);
    match machine
        .memory_mut()
        .copy_large(memory_offset, code_offset, len, &code)
    {
        Ok(()) => (),
        Err(e) => return Control::Exit(e.into()),
    };

    Control::Continue
}

pub fn returndatasize<SPEC: Spec>(machine: &mut Machine) -> Control {
    check!(SPEC::enabled(BYZANTINE)); // EIP-211: New opcodes: RETURNDATASIZE and RETURNDATACOPY
    gas!(machine, gas::BASE);

    let size = U256::from(machine.return_data_buffer.len());
    push_u256!(machine, size);

    Control::Continue
}

pub fn returndatacopy<SPEC: Spec>(machine: &mut Machine) -> Control {
    check!(SPEC::enabled(BYZANTINE)); // EIP-211: New opcodes: RETURNDATASIZE and RETURNDATACOPY
    pop_u256!(machine, memory_offset, data_offset, len);
    gas_or_fail!(machine, gas::verylowcopy_cost(len));
    memory_resize!(machine, memory_offset, len);
    if data_offset
        .checked_add(len)
        .map(|l| l > U256::from(machine.return_data_buffer.len()))
        .unwrap_or(true)
    {
        return Control::Exit(ExitError::OutOfOffset.into());
    }

    match machine
        .memory
        .copy_large(memory_offset, data_offset, len, &machine.return_data_buffer)
    {
        Ok(()) => Control::Continue,
        Err(e) => Control::Exit(e.into()),
    }
}

pub fn blockhash<H: Handler>(machine: &mut Machine, handler: &mut H) -> Control {
    gas!(machine, gas::BLOCKHASH);

    pop_u256!(machine, number);
    push!(machine, handler.block_hash(number));

    Control::Continue
}

pub fn coinbase<H: Handler>(machine: &mut Machine, handler: &mut H) -> Control {
    gas!(machine, gas::BASE);

    push!(machine, handler.env().block_coinbase.into());
    Control::Continue
}

pub fn timestamp<H: Handler>(machine: &mut Machine, handler: &mut H) -> Control {
    gas!(machine, gas::BASE);
    push_u256!(machine, handler.env().block_timestamp);
    Control::Continue
}

pub fn number<H: Handler>(machine: &mut Machine, handler: &mut H) -> Control {
    gas!(machine, gas::BASE);

    push_u256!(machine, handler.env().block_number);
    Control::Continue
}

pub fn difficulty<H: Handler>(machine: &mut Machine, handler: &mut H) -> Control {
    gas!(machine, gas::BASE);

    push_u256!(machine, handler.env().block_difficulty);
    Control::Continue
}

pub fn gaslimit<H: Handler>(machine: &mut Machine, handler: &mut H) -> Control {
    gas!(machine, gas::BASE);

    push_u256!(machine, handler.env().block_gas_limit);
    Control::Continue
}

pub fn sload<H: Handler, SPEC: Spec>(machine: &mut Machine, handler: &mut H) -> Control {
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
    Control::Continue
}

pub fn sstore<H: Handler, SPEC: Spec>(machine: &mut Machine, handler: &mut H) -> Control {
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
    let remaining_gas = machine.gas.remaining();
    gas_or_fail!(
        machine,
        gas::sstore_cost::<SPEC>(original, old, new, remaining_gas, is_cold)
    );
    refund!(machine, gas::sstore_refund::<SPEC>(original, old, new));
    Control::Continue
}

pub fn gas(machine: &mut Machine) -> Control {
    gas!(machine, gas::BASE);

    push_u256!(machine, U256::from(machine.gas.remaining()));
    Control::Continue
}

pub fn log<H: Handler, SPEC: Spec>(machine: &mut Machine, n: u8, handler: &mut H) -> Control {
    check!(!SPEC::IS_STATIC_CALL);

    pop_u256!(machine, offset, len);
    gas_or_fail!(machine, gas::log_cost(n, len));
    memory_resize!(machine, offset, len);
    let data = if len == U256::zero() {
        Bytes::new()
    } else {
        let offset = as_usize_or_fail!(offset);
        let len = as_usize_or_fail!(len);

        Bytes::from(machine.memory().get(offset, len))
    };

    let mut topics = Vec::new();
    for _ in 0..(n as usize) {
        match machine.stack_mut().pop() {
            Ok(value) => {
                topics.push(value);
            }
            Err(e) => return Control::Exit(e.into()),
        }
    }

    handler.log(machine.contract.address, topics, data);
    Control::Continue
}

pub fn selfdestruct<H: Handler, SPEC: Spec>(machine: &mut Machine, handler: &mut H) -> Control {
    check!(!SPEC::IS_STATIC_CALL);
    pop!(machine, target);

    let res = handler.selfdestruct(machine.contract.address, target.into());
    inspect!(handler, selfdestruct);

    if !res.previously_destroyed {
        refund!(machine, gas::SELFDESTRUCT)
    }
    gas!(machine, gas::selfdestruct_cost::<SPEC>(res));

    Control::Exit(ExitSucceed::SelfDestructed.into())
}

#[inline]
fn gas_call_l64_after<SPEC: Spec>(machine: &mut Machine) -> Result<u64, ExitReason> {
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
) -> Control {
    check!(!SPEC::IS_STATIC_CALL);
    if is_create2 {
        check!(SPEC::enabled(CONSTANTINOPLE)); // EIP-1014: Skinny CREATE2
    }

    machine.return_data_buffer = Bytes::new();

    pop_u256!(machine, value, code_offset, len);

    memory_resize!(machine, code_offset, len);
    let code = if len == U256::zero() {
        Bytes::new()
    } else {
        let code_offset = as_usize_or_fail!(code_offset);
        let len = as_usize_or_fail!(len);

        machine.memory().get(code_offset, len)
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
    let created_address: H256 = if matches!(reason, ExitReason::Succeed(_)) {
        address.map(|a| a.into()).unwrap_or_default()
    } else {
        H256::default()
    };
    inspect!(handler, create_return, created_address);
    push!(machine, created_address);
    // reimburse gas that is not spend
    machine.gas.reimburse_unspend(&reason, gas);
    match reason {
        ExitReason::Fatal(e) => Control::Exit(e.into()),
        _ => Control::Continue,
    }
}

pub fn call<H: Handler, SPEC: Spec>(
    machine: &mut Machine,
    scheme: CallScheme,
    handler: &mut H,
) -> Control {
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
            if SPEC::IS_STATIC_CALL && value != U256::zero() {
                return Control::Exit(ExitReason::Error(ExitError::CallNotAllowedInsideStatic));
            }
            value
        }
        CallScheme::DelegateCall | CallScheme::StaticCall => U256::zero(),
    };

    pop_u256!(machine, in_offset, in_len, out_offset, out_len);

    memory_resize!(machine, in_offset, in_len);
    memory_resize!(machine, out_offset, out_len);

    let input = if in_len == U256::zero() {
        Bytes::new()
    } else {
        let in_offset = as_usize_or_fail!(in_offset);
        let in_len = as_usize_or_fail!(in_len);

        machine.memory().get(in_offset, in_len)
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
    if matches!(scheme, CallScheme::Call | CallScheme::CallCode) && transfer.value != U256::zero() {
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

    inspect!(handler, call_return, reason.clone());
    let target_len = min(out_len, U256::from(machine.return_data_buffer.len()));
    // return unspend gas.
    machine.gas.reimburse_unspend(&reason, gas);
    match reason {
        ExitReason::Succeed(_) => {
            match machine.memory.copy_large(
                out_offset,
                U256::zero(),
                target_len,
                &machine.return_data_buffer,
            ) {
                Ok(()) => {
                    push_u256!(machine, U256::one());
                    Control::Continue
                }
                Err(_) => {
                    push_u256!(machine, U256::zero());
                    Control::Continue
                }
            }
        }
        ExitReason::Revert(_) => {
            push_u256!(machine, U256::zero());
            let _ = machine.memory.copy_large(
                out_offset,
                U256::zero(),
                target_len,
                &machine.return_data_buffer,
            );
            Control::Continue
        }
        ExitReason::Error(_) => {
            push_u256!(machine, U256::zero());
            Control::Continue
        }
        ExitReason::Fatal(e) => {
            push_u256!(machine, U256::zero());
            Control::Exit(e.into())
        }
    }
}
