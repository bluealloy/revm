use super::Control;
use crate::{
    error::{ExitError, ExitFatal, ExitReason, ExitSucceed},
    CallScheme, CallContext, CreateScheme, ExtHandler, Machine, Transfer,
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

    try_or_fail!(machine.memory_mut().resize_offset(from, len));
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

pub fn chainid<H: ExtHandler>(machine: &mut Machine, handler: &mut H) -> Control {
    push_u256!(machine, handler.chain_id());

    Control::Continue
}

pub fn address(machine: &mut Machine) -> Control {
    let ret = H256::from(machine.contract.address);
    push!(machine, ret);

    Control::Continue
}

pub fn balance<H: ExtHandler>(machine: &mut Machine, handler: &mut H) -> Control {
    pop!(machine, address);
    push_u256!(machine, handler.balance(address.into()).0);

    Control::Continue
}

pub fn selfbalance<H: ExtHandler>(machine: &mut Machine, handler: &mut H) -> Control {
    push_u256!(machine, handler.balance(machine.contract.address).0);

    Control::Continue
}

pub fn origin<H: ExtHandler>(machine: &mut Machine, handler: &mut H) -> Control {
    let ret = H256::from(handler.origin());
    push!(machine, ret);

    Control::Continue
}

pub fn caller(machine: &mut Machine) -> Control {
    let ret = H256::from(machine.contract.caller);
    push!(machine, ret);

    Control::Continue
}

pub fn callvalue(machine: &mut Machine) -> Control {
    let mut ret = H256::default();
    machine.contract.value.to_big_endian(&mut ret[..]);
    push!(machine, ret);

    Control::Continue
}

pub fn gasprice<H: ExtHandler>(machine: &mut Machine, handler: &mut H) -> Control {
    let mut ret = H256::default();
    handler.gas_price().to_big_endian(&mut ret[..]);
    push!(machine, ret);

    Control::Continue
}

pub fn extcodesize<H: ExtHandler>(machine: &mut Machine, handler: &mut H) -> Control {
    pop!(machine, address);
    push_u256!(machine, handler.code_size(address.into()).0);

    Control::Continue
}

pub fn extcodehash<H: ExtHandler>(machine: &mut Machine, handler: &mut H) -> Control {
    pop!(machine, address);
    push!(machine, handler.code_hash(address.into()).0);

    Control::Continue
}

pub fn extcodecopy<H: ExtHandler>(machine: &mut Machine, handler: &mut H) -> Control {
    pop!(machine, address);
    pop_u256!(machine, memory_offset, code_offset, len);

    try_or_fail!(machine.memory_mut().resize_offset(memory_offset, len));
    match machine.memory_mut().copy_large(
        memory_offset,
        code_offset,
        len,
        &handler.code(address.into()).0,
    ) {
        Ok(()) => (),
        Err(e) => return Control::Exit(e.into()),
    };

    Control::Continue
}

pub fn returndatasize(machine: &mut Machine) -> Control {
    let size = U256::from(machine.return_data_buffer.len());
    push_u256!(machine, size);

    Control::Continue
}

pub fn returndatacopy(machine: &mut Machine) -> Control {
    pop_u256!(machine, memory_offset, data_offset, len);

    try_or_fail!(machine.memory_mut().resize_offset(memory_offset, len));
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
    pop_u256!(machine, number);
    push!(machine, handler.block_hash(number));

    Control::Continue
}

pub fn coinbase<H: ExtHandler>(machine: &mut Machine, handler: &mut H) -> Control {
    push!(machine, handler.block_coinbase().into());
    Control::Continue
}

pub fn timestamp<H: ExtHandler>(machine: &mut Machine, handler: &mut H) -> Control {
    push_u256!(machine, handler.block_timestamp());
    Control::Continue
}

pub fn number<H: ExtHandler>(machine: &mut Machine, handler: &mut H) -> Control {
    push_u256!(machine, handler.block_number());
    Control::Continue
}

pub fn difficulty<H: ExtHandler>(machine: &mut Machine, handler: &mut H) -> Control {
    push_u256!(machine, handler.block_difficulty());
    Control::Continue
}

pub fn gaslimit<H: ExtHandler>(machine: &mut Machine, handler: &mut H) -> Control {
    push_u256!(machine, handler.block_gas_limit());
    Control::Continue
}

pub fn sload<H: ExtHandler, const OPCODE_TRACE: bool>(
    machine: &mut Machine,
    handler: &mut H,
) -> Control {
    pop!(machine, index);
    let value = handler.sload(machine.contract.address, index);
    push!(machine, value.0);
    // if OPCODE_TRACE {
    // 	event!(SLoad {
    // 		address: machine.contract.address,
    // 		index,
    // 		value
    // 	});
    // }

    Control::Continue
}

pub fn sstore<H: ExtHandler, const OPCODE_TRACE: bool>(
    machine: &mut Machine,
    handler: &mut H,
) -> Control {
    pop!(machine, index, value);
    // if OPCODE_TRACE {
    // 	event!(SStore {
    // 		address: machine.contract.address,
    // 		index,
    // 		value
    // 	});
    // }

    handler.sstore(machine.contract.address, index, value);
    Control::Continue
}

pub fn gas<H: ExtHandler>(machine: &mut Machine, handler: &mut H) -> Control {
    push_u256!(machine, handler.gas_left());

    Control::Continue
}

pub fn log<H: ExtHandler>(machine: &mut Machine, n: u8, handler: &mut H) -> Control {
    pop_u256!(machine, offset, len);

    try_or_fail!(machine.memory_mut().resize_offset(offset, len));
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

pub fn suicide<H: ExtHandler, const CALL_TRACE: bool>(
    machine: &mut Machine,
    handler: &mut H,
) -> Control {
    pop!(machine, target);

    match handler.mark_delete::<CALL_TRACE>(machine.contract.address, target.into()) {
        Ok(()) => (),
        Err(e) => return Control::Exit(e.into()),
    }

    Control::Exit(ExitSucceed::Suicided.into())
}

pub fn create<
    H: ExtHandler,
    const CALL_TRACE: bool,
    const GAS_TRACE: bool,
    const OPCODE_TRACE: bool,
>(
    machine: &mut Machine,
    is_create2: bool,
    handler: &mut H,
) -> Control {
    machine.return_data_buffer = Bytes::new();

    pop_u256!(machine, value, code_offset, len);

    try_or_fail!(machine.memory_mut().resize_offset(code_offset, len));
    let code = if len == U256::zero() {
        Bytes::new()
    } else {
        let code_offset = as_usize_or_fail!(code_offset);
        let len = as_usize_or_fail!(len);

        machine.memory().get(code_offset, len)
    };
    let scheme = if is_create2 {
        pop!(machine, salt);
        let code_hash = H256::from_slice(Keccak256::digest(&code).as_slice());
        CreateScheme::Create2 {
            caller: machine.contract.address,
            salt,
            code_hash,
        }
    } else {
        let (nonce,_) = handler.nonce(machine.contract.address);
        CreateScheme::Legacy {
            caller: machine.contract.address,
            nonce,
        }
    };

    let (reason, address, return_data) = handler.create::<CALL_TRACE, GAS_TRACE, OPCODE_TRACE>(
        machine.contract.address,
        scheme,
        value,
        code,
        None,
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

pub fn call<
    H: ExtHandler,
    const CALL_TRACE: bool,
    const GAS_TRACE: bool,
    const OPCODE_TRACE: bool,
>(
    machine: &mut Machine,
    scheme: CallScheme,
    handler: &mut H,
) -> Control {
    machine.return_data_buffer = Bytes::new();

    pop_u256!(machine, gas);
    pop!(machine, to);
    let gas = if gas > U256::from(u64::MAX) {
        None
    } else {
        Some(gas.as_u64())
    };

    let value = match scheme {
        CallScheme::Call | CallScheme::CallCode => {
            pop_u256!(machine, value);
            value
        }
        CallScheme::DelegateCall | CallScheme::StaticCall => U256::zero(),
    };

    pop_u256!(machine, in_offset, in_len, out_offset, out_len);

    try_or_fail!(machine.memory_mut().resize_offset(in_offset, in_len));
    try_or_fail!(machine.memory_mut().resize_offset(out_offset, out_len));

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

    let (reason, return_data) = handler.call::<CALL_TRACE, GAS_TRACE, OPCODE_TRACE>(
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
