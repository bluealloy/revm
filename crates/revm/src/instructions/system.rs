use super::gas;
use crate::{
    machine::Machine, CallContext, CallScheme, CreateScheme, Host, Return, Spec, Transfer,
};
// 	CallScheme, Capture, CallContext, CreateScheme, ,
// 	, Runtime, Transfer,
// };
use crate::{alloc::vec::Vec, instructions::macros::as_usize_or_fail, spec::SpecId::*};
use bytes::Bytes;
use core::cmp::min;
use primitive_types::{H256, U256};
use sha3::{Digest, Keccak256};

pub fn sha3(machine: &mut Machine) -> Result<(), Return> {
    let (from, len) = machine.stack.pop2()?;
    gas_or_fail!(machine, gas::sha3_cost(len));
    let len = as_usize_or_fail(&len, Return::OutOfGas)?;
    let data = if len == 0 {
        Bytes::new()
        // TODO optimization, we can return hadrcoded value of keccak256:digest(&[])
    } else {
        let from = as_usize_or_fail(&from, Return::OutOfGas)?;
        memory_resize!(machine, from, len);
        Bytes::copy_from_slice(machine.memory.get_slice(from, len))
    };

    let ret = Keccak256::digest(data.as_ref());
    machine.stack.push_h256(H256::from_slice(ret.as_slice()))?;
    Ok(())
}

pub fn chainid<H: Host, SPEC: Spec>(machine: &mut Machine, host: &mut H) -> Result<(), Return> {
    SPEC::require(ISTANBUL)?; // EIP-1344: ChainID opcode
                              //gas!(machine, gas::BASE);

    machine.stack.push(host.env().cfg.chain_id)?;
    Ok(())
}

pub fn address(machine: &mut Machine) -> Result<(), Return> {
    //gas!(machine, gas::BASE);

    let ret = H256::from(machine.contract.address);
    machine.stack.push_h256(ret)?;
    Ok(())
}

pub fn balance<H: Host, SPEC: Spec>(machine: &mut Machine, host: &mut H) -> Result<(), Return> {
    let address = machine.stack.pop_address()?;
    let (balance, is_cold) = host.balance(address);
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
    machine.stack.push_unchecked(balance);
    Ok(())
}

pub fn selfbalance<H: Host, SPEC: Spec>(machine: &mut Machine, host: &mut H) -> Result<(), Return> {
    SPEC::require(ISTANBUL)?; // EIP-1884: Repricing for trie-size-dependent opcodes
                              //gas!(machine, gas::LOW);
    let (balance, _) = host.balance(machine.contract.address);
    machine.stack.push(balance)?;
    Ok(())
}

pub fn basefee<H: Host, SPEC: Spec>(machine: &mut Machine, host: &mut H) -> Result<(), Return> {
    SPEC::require(LONDON)?; // EIP-3198: BASEFEE opcode
                            //gas!(machine, gas::BASE);
    machine.stack.push(host.env().block.basefee)?;
    Ok(())
}

pub fn origin<H: Host>(machine: &mut Machine, host: &mut H) -> Result<(), Return> {
    //gas!(machine, gas::BASE);

    let ret = H256::from(host.env().tx.caller);
    machine.stack.push_h256(ret)?;
    Ok(())
}

pub fn caller(machine: &mut Machine) -> Result<(), Return> {
    //gas!(machine, gas::BASE);

    let ret = H256::from(machine.contract.caller);
    machine.stack.push_h256(ret)?;
    Ok(())
}

pub fn callvalue(machine: &mut Machine) -> Result<(), Return> {
    //gas!(machine, gas::BASE);

    let mut ret = H256::default();
    machine.contract.value.to_big_endian(&mut ret[..]);
    machine.stack.push_h256(ret)?;
    Ok(())
}

pub fn gasprice<H: Host>(machine: &mut Machine, host: &mut H) -> Result<(), Return> {
    //gas!(machine, gas::BASE);
    machine.stack.push(host.env().effective_gas_price())?;
    Ok(())
}

pub fn extcodesize<H: Host, SPEC: Spec>(machine: &mut Machine, host: &mut H) -> Result<(), Return> {
    let address = machine.stack.pop_address()?;

    let (code, is_cold) = host.code(address);
    gas!(machine, gas::account_access_gas::<SPEC>(is_cold));

    machine.stack.push_unchecked(U256::from(code.len()));
    Ok(())
}

pub fn extcodehash<H: Host, SPEC: Spec>(machine: &mut Machine, host: &mut H) -> Result<(), Return> {
    SPEC::require(CONSTANTINOPLE)?; // EIP-1052: EXTCODEHASH opcode
    let address = machine.stack.pop_address()?;
    let (code_hash, is_cold) = host.code_hash(address);
    gas!(
        machine,
        if SPEC::enabled(ISTANBUL) {
            // EIP-1884: Repricing for trie-size-dependent opcodes
            gas::account_access_gas::<SPEC>(is_cold)
        } else {
            400
        }
    );
    // XXX skip bounds check
    machine.stack.push_h256(code_hash)?;
    Ok(())
}

pub fn extcodecopy<H: Host, SPEC: Spec>(machine: &mut Machine, host: &mut H) -> Result<(), Return> {
    let address = machine.stack.pop_address()?;
    let (memory_offset, code_offset, len_u256) = machine.stack.pop3()?;

    let (code, is_cold) = host.code(address);
    gas_or_fail!(machine, gas::extcodecopy_cost::<SPEC>(len_u256, is_cold));
    let len = as_usize_or_fail(&len_u256, Return::OutOfGas)?;
    if len == 0 {
        return Ok(());
    }
    let memory_offset = as_usize_or_fail(&memory_offset, Return::OutOfGas)?;
    let code_offset = min(as_usize_saturated!(code_offset), code.len());
    memory_resize!(machine, memory_offset, len);

    machine
        .memory
        .set_data(memory_offset, code_offset, len, &code);
    Ok(())
}

pub fn returndatasize<SPEC: Spec>(machine: &mut Machine) -> Result<(), Return> {
    SPEC::require(BYZANTINE)?; // EIP-211: New opcodes: RETURNDATASIZE and RETURNDATACOPY
                               //gas!(machine, gas::BASE);

    let size = U256::from(machine.return_data_buffer.len());
    machine.stack.push(size)?;
    Ok(())
}

pub fn returndatacopy<SPEC: Spec>(machine: &mut Machine) -> Result<(), Return> {
    SPEC::require(BYZANTINE)?; // EIP-211: New opcodes: RETURNDATASIZE and RETURNDATACOPY
    let (memory_offset, offset, len) = machine.stack.pop3()?;
    gas_or_fail!(machine, gas::verylowcopy_cost(len));
    let len = as_usize_or_fail(&len, Return::OutOfGas)?;
    let memory_offset = as_usize_or_fail(&memory_offset, Return::OutOfGas)?;
    let data_offset = as_usize_saturated!(offset);
    memory_resize!(machine, memory_offset, len);
    let (data_end, overflow) = data_offset.overflowing_add(len);
    if overflow || data_end > machine.return_data_buffer.len() {
        return Err(Return::OutOfOffset);
    }

    machine.memory.set(
        memory_offset,
        &machine.return_data_buffer[data_offset..data_end],
    );
    Ok(())
}

pub fn blockhash<H: Host>(machine: &mut Machine, host: &mut H) -> Result<(), Return> {
    //gas!(machine, gas::BLOCKHASH);

    let number = machine.stack.pop()?;
    machine.stack.push_h256(host.block_hash(number))?;
    Ok(())
}

pub fn coinbase<H: Host>(machine: &mut Machine, host: &mut H) -> Result<(), Return> {
    //gas!(machine, gas::BASE);

    machine.stack.push_h256(host.env().block.coinbase.into())?;
    Ok(())
}

pub fn timestamp<H: Host>(machine: &mut Machine, host: &mut H) -> Result<(), Return> {
    //gas!(machine, gas::BASE);
    machine.stack.push(host.env().block.timestamp)?;
    Ok(())
}

pub fn number<H: Host>(machine: &mut Machine, host: &mut H) -> Result<(), Return> {
    //gas!(machine, gas::BASE);

    machine.stack.push(host.env().block.number)?;
    Ok(())
}

pub fn difficulty<H: Host>(machine: &mut Machine, host: &mut H) -> Result<(), Return> {
    //gas!(machine, gas::BASE);

    machine.stack.push(host.env().block.difficulty)?;
    Ok(())
}

pub fn gaslimit<H: Host>(machine: &mut Machine, host: &mut H) -> Result<(), Return> {
    //gas!(machine, gas::BASE);

    machine.stack.push(host.env().block.gas_limit)?;
    Ok(())
}

pub fn sload<H: Host, SPEC: Spec>(machine: &mut Machine, host: &mut H) -> Result<(), Return> {
    let index = machine.stack.pop()?;
    let (value, is_cold) = host.sload(machine.contract.address, index);
    gas!(machine, gas::sload_cost::<SPEC>(is_cold));
    machine.stack.push(value)?;
    Ok(())
}

pub fn sstore<H: Host, SPEC: Spec>(machine: &mut Machine, host: &mut H) -> Result<(), Return> {
    SPEC::err_if_static_call()?;

    let (index, value) = machine.stack.pop2()?;
    let (original, old, new, is_cold) = host.sstore(machine.contract.address, index, value);
    gas_or_fail!(machine, {
        let remaining_gas = machine.gas.remaining();
        gas::sstore_cost::<SPEC>(original, old, new, remaining_gas, is_cold)
    });
    refund!(machine, gas::sstore_refund::<SPEC>(original, old, new));
    Ok(())
}

pub fn gas(machine: &mut Machine) -> Result<(), Return> {
    //gas!(machine, gas::BASE);

    machine.stack.push(U256::from(machine.gas.remaining()))?;
    machine.add_next_gas_block(machine.program_counter - 1)
}

pub fn log<H: Host, SPEC: Spec>(machine: &mut Machine, n: u8, host: &mut H) -> Result<(), Return> {
    SPEC::err_if_static_call()?;

    let (offset, len) = machine.stack.pop2()?;
    gas_or_fail!(machine, gas::log_cost(n, len));
    let len = as_usize_or_fail(&len, Return::OutOfGas)?;
    let data = if len == 0 {
        Bytes::new()
    } else {
        let offset = as_usize_or_fail(&offset, Return::OutOfGas)?;
        memory_resize!(machine, offset, len);
        Bytes::copy_from_slice(machine.memory.get_slice(offset, len))
    };
    let n = n as usize;
    if machine.stack.len() < n {
        return Err(Return::StackUnderflow);
    }

    let mut topics = Vec::with_capacity(n);
    for _ in 0..(n) {
        let mut t = H256::zero();
        machine.stack.pop().unwrap().to_big_endian(t.as_bytes_mut());
        topics.push(t);
    }

    host.log(machine.contract.address, topics, data);
    Ok(())
}

pub fn selfdestruct<H: Host, SPEC: Spec>(
    machine: &mut Machine,
    host: &mut H,
) -> Result<(), Return> {
    SPEC::err_if_static_call()?;
    let target = machine.stack.pop_address()?;

    let res = host.selfdestruct(machine.contract.address, target);

    // EIP-3529: Reduction in refunds
    if !SPEC::enabled(LONDON) && !res.previously_destroyed {
        refund!(machine, gas::SELFDESTRUCT)
    }
    gas!(machine, gas::selfdestruct_cost::<SPEC>(res));

    Err(Return::SelfDestruct)
}

fn gas_call_l64_after<SPEC: Spec>(machine: &mut Machine) -> Result<u64, Return> {
    if SPEC::enabled(TANGERINE) {
        //EIP-150: Gas cost changes for IO-heavy operations
        let gas = machine.gas().remaining();
        Ok(gas - gas / 64)
    } else {
        Ok(machine.gas().remaining())
    }
}

pub fn create<H: Host, SPEC: Spec>(
    machine: &mut Machine,
    is_create2: bool,
    host: &mut H,
) -> Result<(), Return> {
    SPEC::err_if_static_call()?;
    if is_create2 {
        SPEC::require(CONSTANTINOPLE)?; // EIP-1014: Skinny CREATE2
    }

    machine.return_data_buffer = Bytes::new();

    let (value, code_offset, len) = machine.stack.pop3()?;
    let len = as_usize_or_fail(&len, Return::OutOfGas)?;

    let code = if len == 0 {
        Bytes::new()
    } else {
        let code_offset = as_usize_or_fail(&code_offset, Return::OutOfGas)?;
        memory_resize!(machine, code_offset, len);
        Bytes::copy_from_slice(machine.memory.get_slice(code_offset, len))
    };

    let scheme = if is_create2 {
        let salt = machine.stack.pop()?;
        gas_or_fail!(machine, gas::create2_cost(len));
        CreateScheme::Create2 { salt }
    } else {
        gas!(machine, gas::CREATE);
        CreateScheme::Create
    };

    // take remaining gas and deduce l64 part of it.
    let gas_limit = gas_call_l64_after::<SPEC>(machine)?;
    gas!(machine, gas_limit);

    let (reason, address, gas, return_data) =
        host.create::<SPEC>(machine.contract.address, scheme, value, code, gas_limit);
    machine.return_data_buffer = return_data;
    let created_address: H256 = if reason.is_normal() {
        address.map(|a| a.into()).unwrap_or_default()
    } else {
        H256::default()
    };
    machine.stack.push_h256(created_address)?;
    // reimburse gas that is not spend
    machine.gas.reimburse_unspend(&reason, gas);
    match reason {
        Return::FatalNotSupported => Err(Return::FatalNotSupported),
        _ => machine.add_next_gas_block(machine.program_counter - 1),
    }
}

pub fn call<H: Host, SPEC: Spec>(
    machine: &mut Machine,
    scheme: CallScheme,
    host: &mut H,
) -> Result<(), Return> {
    match scheme {
        CallScheme::DelegateCall => SPEC::require(HOMESTEAD)?, // EIP-7: DELEGATECALL
        CallScheme::StaticCall => SPEC::require(BYZANTINE)?,   // EIP-214: New opcode STATICCALL
        _ => (),
    }
    machine.return_data_buffer = Bytes::new();

    let local_gas_limit = machine.stack.pop()?;
    let to = machine.stack.pop_address()?;
    let local_gas_limit = if local_gas_limit > U256::from(u64::MAX) {
        u64::MAX
    } else {
        local_gas_limit.as_u64()
    };

    let value = match scheme {
        CallScheme::CallCode => machine.stack.pop()?,
        CallScheme::Call => {
            let value = machine.stack.pop()?;
            if SPEC::IS_STATIC_CALL && !value.is_zero() {
                return Err(Return::CallNotAllowedInsideStatic);
            }
            value
        }
        CallScheme::DelegateCall | CallScheme::StaticCall => U256::zero(),
    };

    let (in_offset, in_len, out_offset, out_len) = machine.stack.pop4()?;

    let in_len = as_usize_or_fail(&in_len, Return::OutOfGas)?;
    let input = if in_len != 0 {
        let in_offset = as_usize_or_fail(&in_offset, Return::OutOfGas)?;
        memory_resize!(machine, in_offset, in_len);
        Bytes::copy_from_slice(machine.memory.get_slice(in_offset, in_len))
    } else {
        Bytes::new()
    };

    let out_len = as_usize_or_fail(&out_len, Return::OutOfGas)?;
    let out_offset = if out_len != 0 {
        let out_offset = as_usize_or_fail(&out_offset, Return::OutOfGas)?;
        memory_resize!(machine, out_offset, out_len);
        out_offset
    } else {
        usize::MAX //unrealistic value so we are sure it is not used
    };

    let context = match scheme {
        CallScheme::Call | CallScheme::StaticCall => CallContext {
            address: to,
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
            target: to,
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

    // load account and calculate gas cost.
    let (is_cold, exist) = host.load_account(to);
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
    let global_gas_limit = gas_call_l64_after::<SPEC>(machine)?;
    let mut gas_limit = min(global_gas_limit, local_gas_limit);

    gas!(machine, gas_limit);

    // add call stipend if there is value to be transfered.
    if matches!(scheme, CallScheme::Call | CallScheme::CallCode) && !transfer.value.is_zero() {
        gas_limit = gas_limit.saturating_add(gas::CALL_STIPEND);
    }
    let is_static = matches!(scheme, CallScheme::StaticCall);

    // CALL CONTRACT, with static or ordinary spec.
    let (reason, gas, return_data) = if is_static {
        host.call::<SPEC::STATIC>(to, transfer, input, gas_limit, context)
    } else {
        host.call::<SPEC>(to, transfer, input, gas_limit, context)
    };
    machine.return_data_buffer = return_data;

    let target_len = min(out_len, machine.return_data_buffer.len());
    // return unspend gas.
    machine.gas.reimburse_unspend(&reason, gas);
    if reason.is_normal() {
        machine
            .memory
            .set(out_offset, &machine.return_data_buffer[..target_len]);
        machine.stack.push(U256::one())?;
    } else if reason.is_revert() {
        machine.stack.push(U256::zero())?;
        machine
            .memory
            .set(out_offset, &machine.return_data_buffer[..target_len]);
    } else {
        machine.stack.push(U256::zero())?;
    }

    machine.add_next_gas_block(machine.program_counter - 1)
}
