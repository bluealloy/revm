use super::gas;
use crate::{
    instructions::macros::as_usize_or_fail, machine::Machine, util, Return, Spec, SpecId::*,
};
use primitive_types::{H256, U256};

pub fn codesize(machine: &mut Machine) -> Result<(), Return> {
    //gas!(machine, gas::BASE);
    let size = U256::from(machine.contract.code_size);
    machine.stack.push(size)?;
    Ok(())
}

pub fn codecopy(machine: &mut Machine) -> Result<(), Return> {
    let (memory_offset, code_offset, len) = machine.stack.pop3()?;
    gas_or_fail!(machine, gas::verylowcopy_cost(len));
    let len = as_usize_or_fail(&len, Return::OutOfGas)?;
    if len == 0 {
        return Ok(());
    }
    let memory_offset = as_usize_or_fail(&memory_offset, Return::OutOfGas)?;
    let code_offset = as_usize_saturated!(code_offset);
    memory_resize!(machine, memory_offset, len);

    machine
        .memory
        .set_data(memory_offset, code_offset, len, &machine.contract.code);

    Ok(())
}

pub fn calldataload(machine: &mut Machine) -> Result<(), Return> {
    //gas!(machine, gas::VERYLOW);

    let index = machine.stack.pop()?;

    let mut load = [0u8; 32];
    #[allow(clippy::needless_range_loop)]
    for i in 0..32 {
        if let Some(p) = index.checked_add(U256::from(i)) {
            if p <= U256::from(usize::MAX) {
                let p = p.as_usize();
                if p < machine.contract.input.len() {
                    load[i] = machine.contract.input[p];
                }
            }
        }
    }

    machine.stack.push_h256(H256::from(load))?;
    Ok(())
}

pub fn calldatasize(machine: &mut Machine) -> Result<(), Return> {
    //gas!(machine, gas::BASE);

    let len = U256::from(machine.contract.input.len());
    machine.stack.push(len)?;
    Ok(())
}

pub fn calldatacopy(machine: &mut Machine) -> Result<(), Return> {
    let (memory_offset, data_offset, len) = machine.stack.pop3()?;
    gas_or_fail!(machine, gas::verylowcopy_cost(len));
    let len = as_usize_or_fail(&len, Return::OutOfGas)?;
    if len == 0 {
        return Ok(());
    }
    let memory_offset = as_usize_or_fail(&memory_offset, Return::OutOfGas)?;
    let data_offset = as_usize_saturated!(data_offset);
    memory_resize!(machine, memory_offset, len);

    machine
        .memory
        .set_data(memory_offset, data_offset, len, &machine.contract.input);
    Ok(())
}

#[inline(always)]
pub fn pop(machine: &mut Machine) -> Result<(), Return> {
    //gas!(machine, gas::BASE);
    machine.stack.reduce_one()
}

pub fn mload(machine: &mut Machine) -> Result<(), Return> {
    //gas!(machine, gas::VERYLOW);

    let index = machine.stack.pop()?;
    let index = as_usize_or_fail(&index, Return::OutOfGas)?;
    memory_resize!(machine, index, 32);
    let ret = util::be_to_u256(machine.memory.get_slice(index, 32));
    machine.stack.push_unchecked(ret);
    Ok(())
}

pub fn mstore(machine: &mut Machine) -> Result<(), Return> {
    //gas!(machine, gas::VERYLOW);

    let (index, value) = machine.stack.pop2()?;

    let index = as_usize_or_fail(&index, Return::OutOfGas)?;
    memory_resize!(machine, index, 32);
    machine.memory.set_u256(index, value);
    Ok(())
}

pub fn mstore8(machine: &mut Machine) -> Result<(), Return> {
    //gas!(machine, gas::VERYLOW);

    let (index, value) = machine.stack.pop2()?;

    let index = as_usize_or_fail(&index, Return::OutOfGas)?;
    memory_resize!(machine, index, 1);
    let value = (value.low_u32() & 0xff) as u8;
    machine.memory.set_byte(index, value);
    Ok(())
}

pub fn jump(machine: &mut Machine) -> Result<(), Return> {
    //gas!(machine, gas::MID);

    let dest = machine.stack.pop()?;
    let dest = as_usize_or_fail(&dest, Return::InvalidJump)?;

    if machine.contract.is_valid_jump(dest) {
        machine.program_counter = dest;
        Ok(())
    } else {
        Err(Return::InvalidJump)
    }
}

pub fn jumpi(machine: &mut Machine) -> Result<(), Return> {
    //gas!(machine, gas::HIGH);

    let (dest, value) = machine.stack.pop2()?;

    if !value.is_zero() {
        let dest = as_usize_or_fail(&dest, Return::InvalidJump)?;
        if machine.contract.is_valid_jump(dest) {
            machine.program_counter = dest;
            Ok(())
        } else {
            Err(Return::InvalidJump)
        }
    } else {
        // if we are not doing jump, add next gas block.
        machine.add_next_gas_block(machine.program_counter - 1)
    }
}

pub fn jumpdest(machine: &mut Machine) -> Result<(), Return> {
    gas!(machine, gas::JUMPDEST);
    machine.add_next_gas_block(machine.program_counter - 1)
}

pub fn pc(machine: &mut Machine) -> Result<(), Return> {
    //gas!(machine, gas::BASE);
    machine
        .stack
        .push(U256::from(machine.program_counter - 1))?;
    Ok(())
}

pub fn msize(machine: &mut Machine) -> Result<(), Return> {
    //gas!(machine, gas::BASE);
    machine
        .stack
        .push(U256::from(machine.memory.effective_len()))?;
    Ok(())
}

// code padding is needed for contracts

pub fn push<const N: usize>(machine: &mut Machine) -> Result<(), Return> {
    //gas!(machine, gas::VERYLOW);
    let start = machine.program_counter;
    let ret = machine
        .stack
        .push_slice::<N>(&machine.contract.code[start..start + N]);
    machine.program_counter += N;
    ret
}

pub fn dup<const N: usize>(machine: &mut Machine) -> Result<(), Return> {
    //gas!(machine, gas::VERYLOW);
    machine.stack.dup::<N>()
}

pub fn swap<const N: usize>(machine: &mut Machine) -> Result<(), Return> {
    //gas!(machine, gas::VERYLOW);
    machine.stack.swap::<N>()
}

pub fn ret(machine: &mut Machine) -> Result<(), Return> {
    // zero gas cost gas!(machine,gas::ZERO);
    let (start, len) = machine.stack.pop2()?;
    let len = as_usize_or_fail(&len, Return::OutOfGas)?;
    if len == 0 {
        machine.return_range = usize::MAX..usize::MAX;
    } else {
        let offset = as_usize_or_fail(&start, Return::OutOfGas)?;
        memory_resize!(machine, offset, len);
        machine.return_range = offset..(offset + len);
    }
    Err(Return::Return)
}

pub fn revert<SPEC: Spec>(machine: &mut Machine) -> Result<(), Return> {
    SPEC::require(BYZANTINE)?; // EIP-140: REVERT instruction
                               // zero gas cost gas!(machine,gas::ZERO);
    let (start, len) = machine.stack.pop2()?;
    let len = as_usize_or_fail(&len, Return::OutOfGas)?;
    if len == 0 {
        machine.return_range = usize::MAX..usize::MAX;
    } else {
        let offset = as_usize_or_fail(&start, Return::OutOfGas)?;
        memory_resize!(machine, offset, len);
        machine.return_range = offset..(offset + len);
    }
    Err(Return::Revert)
}
