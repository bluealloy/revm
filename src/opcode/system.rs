use super::{gas, Control};
use crate::{
    error::{ExitError, ExitFatal, ExitReason, ExitSucceed},
    machine::Machine,
    CallContext, CallScheme, CreateScheme, ExtHandler, Spec, Transfer,
};
// 	CallScheme, Capture, CallContext, CreateScheme, ,
// 	, Runtime, Transfer,
// };
use alloc::vec::Vec;
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

pub fn chainid<H: ExtHandler, SPEC: Spec>(machine: &mut Machine, handler: &mut H) -> Control {
    enabled!(SPEC::has_chain_id);
    gas!(machine, gas::BASE);

    push_u256!(machine, handler.chain_id());

    Control::Continue
}

pub fn address(machine: &mut Machine) -> Control {
    gas!(machine, gas::BASE);

    let ret = H256::from(machine.contract.address);
    push!(machine, ret);

    Control::Continue
}

pub fn balance<H: ExtHandler, SPEC: Spec>(machine: &mut Machine, handler: &mut H) -> Control {
    pop!(machine, address);
    let (balance, is_cold) = handler.balance(address.into());
    gas!(
        machine,
        gas::account_access_cost::<SPEC>(is_cold, SPEC::gas_balance)
    );
    push_u256!(machine, balance);

    Control::Continue
}

pub fn selfbalance<H: ExtHandler, SPEC: Spec>(machine: &mut Machine, handler: &mut H) -> Control {
    enabled!(SPEC::has_self_balance);
    let (balance, _) = handler.balance(machine.contract.address);
    gas!(machine, gas::LOW);
    push_u256!(machine, balance);

    Control::Continue
}

pub fn origin<H: ExtHandler>(machine: &mut Machine, handler: &mut H) -> Control {
    gas!(machine, gas::BASE);

    let ret = H256::from(handler.origin());
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

pub fn gasprice<H: ExtHandler>(machine: &mut Machine, handler: &mut H) -> Control {
    gas!(machine, gas::BASE);

    let mut ret = H256::default();
    handler.gas_price().to_big_endian(&mut ret[..]);
    push!(machine, ret);

    Control::Continue
}

pub fn extcodesize<H: ExtHandler, SPEC: Spec>(machine: &mut Machine, handler: &mut H) -> Control {
    pop!(machine, address);

    let (code_size, is_cold) = handler.code_size(address.into());
    gas!(
        machine,
        gas::account_access_cost::<SPEC>(is_cold, SPEC::gas_ext_code)
    );

    push_u256!(machine, code_size);

    Control::Continue
}

pub fn extcodehash<H: ExtHandler, SPEC: Spec>(machine: &mut Machine, handler: &mut H) -> Control {
    enabled!(SPEC::has_ext_code_hash);
    pop!(machine, address);
    let (code_hash, is_cold) = handler.code_hash(address.into());
    gas!(
        machine,
        gas::account_access_cost::<SPEC>(is_cold, SPEC::gas_ext_code_hash)
    );
    push!(machine, code_hash);

    Control::Continue
}

pub fn extcodecopy<H: ExtHandler, SPEC: Spec>(machine: &mut Machine, handler: &mut H) -> Control {
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
    enabled!(SPEC::has_return_data);
    gas!(machine, gas::BASE);

    let size = U256::from(machine.return_data_buffer.len());
    push_u256!(machine, size);

    Control::Continue
}

pub fn returndatacopy<SPEC: Spec>(machine: &mut Machine) -> Control {
    enabled!(SPEC::has_return_data);
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

pub fn blockhash<H: ExtHandler>(machine: &mut Machine, handler: &mut H) -> Control {
    gas!(machine, gas::BLOCKHASH);

    pop_u256!(machine, number);
    push!(machine, handler.block_hash(number));

    Control::Continue
}

pub fn coinbase<H: ExtHandler>(machine: &mut Machine, handler: &mut H) -> Control {
    gas!(machine, gas::BASE);

    push!(machine, handler.block_coinbase().into());
    Control::Continue
}

pub fn timestamp<H: ExtHandler>(machine: &mut Machine, handler: &mut H) -> Control {
    gas!(machine, gas::BASE);
    push_u256!(machine, handler.block_timestamp());
    Control::Continue
}

pub fn number<H: ExtHandler>(machine: &mut Machine, handler: &mut H) -> Control {
    gas!(machine, gas::BASE);

    push_u256!(machine, handler.block_number());
    Control::Continue
}

pub fn difficulty<H: ExtHandler>(machine: &mut Machine, handler: &mut H) -> Control {
    gas!(machine, gas::BASE);

    push_u256!(machine, handler.block_difficulty());
    Control::Continue
}

pub fn gaslimit<H: ExtHandler>(machine: &mut Machine, handler: &mut H) -> Control {
    gas!(machine, gas::BASE);

    push_u256!(machine, handler.block_gas_limit());
    Control::Continue
}

pub fn sload<H: ExtHandler, SPEC: Spec>(machine: &mut Machine, handler: &mut H) -> Control {
    pop!(machine, index);
    let (value, is_cold) = handler.sload(machine.contract.address, index);
    gas!(machine, gas::sload_cost::<SPEC>(is_cold));
    push!(machine, value);
    Control::Continue
}

pub fn sstore<H: ExtHandler, SPEC: Spec>(machine: &mut Machine, handler: &mut H) -> Control {
    enabled!(SPEC::is_not_static_call);

    pop!(machine, index, value);
    let (original, present, new, is_cold) = handler.sstore(machine.contract.address, index, value);

    if SPEC::estimate {
        gas!(machine, SPEC::gas_sstore_set)
    } else {
        let remaining_gas = machine.gas.remaining();
        gas_or_fail!(
            machine,
            gas::sstore_cost::<SPEC>(original, present, new, remaining_gas, is_cold)
        );
        refund!(machine, gas::sstore_refund::<SPEC>(original, present, new));
    }
    Control::Continue
}

pub fn gas(machine: &mut Machine) -> Control {
    gas!(machine, gas::BASE);

    push_u256!(machine, U256::from(machine.gas().left()));
    Control::Continue
}

pub fn log<H: ExtHandler, SPEC: Spec>(machine: &mut Machine, n: u8, handler: &mut H) -> Control {
    enabled!(SPEC::is_not_static_call);

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

pub fn selfdestruct<H: ExtHandler, SPEC: Spec>(machine: &mut Machine, handler: &mut H) -> Control {
    enabled!(SPEC::is_not_static_call);
    pop!(machine, target);

    let res = try_or_fail!(handler.selfdestruct(machine.contract.address, target.into()));

    if !SPEC::estimate && res.previously_destroyed {
        refund!(machine, gas::SELFDESTRUCT)
    }
    gas!(machine, gas::selfdestruct_cost::<SPEC>(res));

    Control::Exit(ExitSucceed::SelfDestructed.into())
}

pub fn create<H: ExtHandler, SPEC: Spec>(
    machine: &mut Machine,
    is_create2: bool,
    handler: &mut H,
) -> Control {
    enabled!(SPEC::is_not_static_call);

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
        //let code_hash = H256::from_slice(Keccak256::digest(&code).as_slice());
        CreateScheme::Create2 { salt }
    } else {
        CreateScheme::Create
    };

    let (reason, address, gas, return_data) = handler.create(
        machine.contract.address,
        scheme,
        value,
        code,
        machine.gas().left(),
    );
    machine.return_data_buffer = return_data;
    let create_address: H256 = address.map(|a| a.into()).unwrap_or_default();

    match reason {
        ExitReason::Succeed(_) => {
            push!(machine, create_address);
            Control::Continue
        }
        ExitReason::Revert(_) => {
            push!(machine, H256::default());
            Control::Continue
        }
        ExitReason::Error(_) => {
            push!(machine, H256::default());
            Control::Continue
        }
        ExitReason::Fatal(e) => {
            push!(machine, H256::default());
            Control::Exit(e.into())
        }
    }
}

pub fn call<H: ExtHandler, SPEC: Spec>(
    machine: &mut Machine,
    scheme: CallScheme,
    handler: &mut H,
) -> Control {
    match scheme {
        CallScheme::Call => enabled!(SPEC::is_not_static_call),
        CallScheme::DelegateCall => enabled!(SPEC::has_delegate_call),
        _ => (),
    }

    machine.return_data_buffer = Bytes::new();

    pop_u256!(machine, gas);
    pop!(machine, to);
    let gas = if gas > U256::from(u64::MAX) {
        u64::MAX
    } else {
        gas.as_u64()
    };

    let value = match scheme {
        CallScheme::Call | CallScheme::CallCode => {
            pop_u256!(machine, value);
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
        Some(Transfer {
            source: machine.contract.address,
            target: to.into(),
            value,
        })
    } else if scheme == CallScheme::CallCode {
        Some(Transfer {
            source: machine.contract.address,
            target: machine.contract.address,
            value,
        })
    } else {
        None
    };

    let (reason, gas, return_data) = handler.call(
        to.into(),
        transfer,
        input,
        gas,
        scheme == CallScheme::StaticCall,
        context,
    );
    machine.return_data_buffer = return_data;
    let target_len = min(out_len, U256::from(machine.return_data_buffer.len()));

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
