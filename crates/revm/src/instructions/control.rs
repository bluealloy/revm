use crate::{gas, interpreter::Interpreter, Return, Spec, SpecId::*};
use primitive_types::U256;

pub fn jump(machine: &mut Interpreter) -> Return {
    // gas!(machine, gas::MID);
    pop!(machine, dest);
    let dest = as_usize_or_fail!(dest, Return::InvalidJump);
    if machine.contract.is_valid_jump(dest) {
        // Safety: In analazis we are checking create our jump table and we do check above to be
        // sure that jump is safe to execute.
        machine.program_counter = unsafe { machine.contract.code.as_ptr().add(dest) };
        Return::Continue
    } else {
        Return::InvalidJump
    }
}

pub fn jumpi(machine: &mut Interpreter) -> Return {
    // gas!(machine, gas::HIGH);
    pop!(machine, dest, value);
    if !value.is_zero() {
        let dest = as_usize_or_fail!(dest, Return::InvalidJump);
        if machine.contract.is_valid_jump(dest) {
            // Safety: In analazis we are checking if jump is valid destination and this if.
            // make this unsafe block safe.
            machine.program_counter = unsafe { machine.contract.code.as_ptr().add(dest) };
            Return::Continue
        } else {
            Return::InvalidJump
        }
    } else {
        // if we are not doing jump, add next gas block.
        machine.add_next_gas_block(machine.program_counter() - 1)
    }
}

pub fn jumpdest(machine: &mut Interpreter) -> Return {
    gas!(machine, gas::JUMPDEST);
    machine.add_next_gas_block(machine.program_counter() - 1)
}

pub fn pc(machine: &mut Interpreter) -> Return {
    // gas!(machine, gas::BASE);
    push!(machine, U256::from(machine.program_counter() - 1));
    Return::Continue
}

pub fn ret(machine: &mut Interpreter) -> Return {
    // zero gas cost gas!(machine,gas::ZERO);
    pop!(machine, start, len);
    let len = as_usize_or_fail!(len, Return::OutOfGas);
    if len == 0 {
        machine.return_range = usize::MAX..usize::MAX;
    } else {
        let offset = as_usize_or_fail!(start, Return::OutOfGas);
        memory_resize!(machine, offset, len);
        machine.return_range = offset..(offset + len);
    }
    Return::Return
}

pub fn revert<SPEC: Spec>(machine: &mut Interpreter) -> Return {
    // zero gas cost gas!(machine,gas::ZERO);
    // EIP-140: REVERT instruction
    check!(SPEC::enabled(BYZANTINE));
    pop!(machine, start, len);
    let len = as_usize_or_fail!(len, Return::OutOfGas);
    if len == 0 {
        machine.return_range = usize::MAX..usize::MAX;
    } else {
        let offset = as_usize_or_fail!(start, Return::OutOfGas);
        memory_resize!(machine, offset, len);
        machine.return_range = offset..(offset + len);
    }
    Return::Revert
}
