use crate::{gas, interpreter::Interpreter, Return, Spec, SpecId::*, U256};

pub fn jump(interp: &mut Interpreter) -> Return {
    // gas!(interp, gas::MID);
    pop!(interp, dest);
    let dest = as_usize_or_fail!(dest, Return::InvalidJump);
    if interp.contract.is_valid_jump(dest) {
        // Safety: In analysis we are checking create our jump table and we do check above to be
        // sure that jump is safe to execute.
        interp.instruction_pointer = unsafe { interp.contract.bytecode.as_ptr().add(dest) };
        Return::Continue
    } else {
        Return::InvalidJump
    }
}

pub fn jumpi(interp: &mut Interpreter) -> Return {
    // gas!(interp, gas::HIGH);
    pop!(interp, dest, value);
    if value != U256::ZERO {
        let dest = as_usize_or_fail!(dest, Return::InvalidJump);
        if interp.contract.is_valid_jump(dest) {
            // Safety: In analysis we are checking if jump is valid destination and
            // this `if` makes this unsafe block safe.
            interp.instruction_pointer = unsafe { interp.contract.bytecode.as_ptr().add(dest) };
            Return::Continue
        } else {
            Return::InvalidJump
        }
    } else {
        // if we are not doing jump, add next gas block.
        interp.add_next_gas_block(interp.program_counter() - 1)
    }
}

pub fn jumpdest(interp: &mut Interpreter) -> Return {
    gas!(interp, gas::JUMPDEST);
    interp.add_next_gas_block(interp.program_counter() - 1)
}

pub fn pc(interp: &mut Interpreter) -> Return {
    // gas!(interp, gas::BASE);
    push!(interp, U256::from(interp.program_counter() - 1));
    Return::Continue
}

pub fn ret(interp: &mut Interpreter) -> Return {
    // zero gas cost gas!(interp,gas::ZERO);
    pop!(interp, start, len);
    let len = as_usize_or_fail!(len, Return::OutOfGas);
    if len == 0 {
        interp.return_range = usize::MAX..usize::MAX;
    } else {
        let offset = as_usize_or_fail!(start, Return::OutOfGas);
        memory_resize!(interp, offset, len);
        interp.return_range = offset..(offset + len);
    }
    Return::Return
}

pub fn revert<SPEC: Spec>(interp: &mut Interpreter) -> Return {
    // zero gas cost gas!(interp,gas::ZERO);
    // EIP-140: REVERT instruction
    check!(SPEC::enabled(BYZANTIUM));
    pop!(interp, start, len);
    let len = as_usize_or_fail!(len, Return::OutOfGas);
    if len == 0 {
        interp.return_range = usize::MAX..usize::MAX;
    } else {
        let offset = as_usize_or_fail!(start, Return::OutOfGas);
        memory_resize!(interp, offset, len);
        interp.return_range = offset..(offset + len);
    }
    Return::Revert
}
