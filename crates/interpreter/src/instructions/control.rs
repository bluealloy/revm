use crate::{
    gas, interpreter::Interpreter, primitives::Spec, primitives::SpecId::*, primitives::U256, Host,
    InstructionResult,
};

pub fn jump(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    // gas!(interp, gas::MID);
    pop!(interpreter, dest);
    let dest = as_usize_or_fail!(interpreter, dest, InstructionResult::InvalidJump);
    if interpreter.contract.is_valid_jump(dest) {
        // Safety: In analysis we are checking create our jump table and we do check above to be
        // sure that jump is safe to execute.
        interpreter.instruction_pointer =
            unsafe { interpreter.contract.bytecode.as_ptr().add(dest) };
    } else {
        interpreter.instruction_result = InstructionResult::InvalidJump;
    }
}

pub fn jumpi(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    // gas!(interp, gas::HIGH);
    pop!(interpreter, dest, value);
    if value != U256::ZERO {
        let dest = as_usize_or_fail!(interpreter, dest, InstructionResult::InvalidJump);
        if interpreter.contract.is_valid_jump(dest) {
            // Safety: In analysis we are checking if jump is valid destination and
            // this `if` makes this unsafe block safe.
            interpreter.instruction_pointer =
                unsafe { interpreter.contract.bytecode.as_ptr().add(dest) };
        } else {
            interpreter.instruction_result = InstructionResult::InvalidJump
        }
    } else if let Some(ret) = interpreter.add_next_gas_block(interpreter.program_counter() - 1) {
        // if we are not doing jump, add next gas block.
        interpreter.instruction_result = ret;
    }
}

pub fn jumpdest(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::JUMPDEST);
    if let Some(ret) = interpreter.add_next_gas_block(interpreter.program_counter() - 1) {
        interpreter.instruction_result = ret;
    }
}

pub fn pc(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    // gas!(interp, gas::BASE);
    push!(interpreter, U256::from(interpreter.program_counter() - 1));
}

pub fn ret(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    // zero gas cost gas!(interp,gas::ZERO);
    pop!(interpreter, start, len);
    let len = as_usize_or_fail!(interpreter, len, InstructionResult::InvalidOperandOOG);
    if len == 0 {
        interpreter.return_range = usize::MAX..usize::MAX;
    } else {
        let offset = as_usize_or_fail!(interpreter, start, InstructionResult::InvalidOperandOOG);
        memory_resize!(interpreter, offset, len);
        interpreter.return_range = offset..(offset + len);
    }
    interpreter.instruction_result = InstructionResult::Return;
}

pub fn revert<SPEC: Spec>(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    // zero gas cost gas!(interp,gas::ZERO);
    // EIP-140: REVERT instruction
    check!(interpreter, SPEC::enabled(BYZANTIUM));
    pop!(interpreter, start, len);
    let len = as_usize_or_fail!(interpreter, len, InstructionResult::InvalidOperandOOG);
    if len == 0 {
        interpreter.return_range = usize::MAX..usize::MAX;
    } else {
        let offset = as_usize_or_fail!(interpreter, start, InstructionResult::InvalidOperandOOG);
        memory_resize!(interpreter, offset, len);
        interpreter.return_range = offset..(offset + len);
    }
    interpreter.instruction_result = InstructionResult::Revert;
}
