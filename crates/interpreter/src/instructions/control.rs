use super::prelude::*;

pub(super) fn jump(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::MID);
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

pub(super) fn jumpi(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::HIGH);
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
    }
}

pub(super) fn jumpdest(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::JUMPDEST);
}

pub(super) fn pc(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, U256::from(interpreter.program_counter() - 1));
}

pub(super) fn ret(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    // zero gas cost
    // gas!(interpreter, gas::ZERO);
    pop!(interpreter, start, len);
    let len = as_usize_or_fail!(interpreter, len);
    if len == 0 {
        interpreter.return_range = usize::MAX..usize::MAX;
    } else {
        let offset = as_usize_or_fail!(interpreter, start);
        memory_resize!(interpreter, offset, len);
        interpreter.return_range = offset..(offset + len);
    }
    interpreter.instruction_result = InstructionResult::Return;
}

// EIP-140: REVERT instruction
pub(super) fn revert(interpreter: &mut Interpreter, _host: &mut dyn Host, spec: SpecId) {
    // zero gas cost
    // gas!(interpreter, gas::ZERO);
    check!(interpreter, SpecId::enabled(spec, BYZANTIUM));
    pop!(interpreter, start, len);
    let len = as_usize_or_fail!(interpreter, len);
    if len == 0 {
        interpreter.return_range = usize::MAX..usize::MAX;
    } else {
        let offset = as_usize_or_fail!(interpreter, start);
        memory_resize!(interpreter, offset, len);
        interpreter.return_range = offset..(offset + len);
    }
    interpreter.instruction_result = InstructionResult::Revert;
}

pub(super) fn stop(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    interpreter.instruction_result = InstructionResult::Stop;
}

pub(super) fn invalid(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    interpreter.instruction_result = InstructionResult::InvalidFEOpcode;
}

pub(super) fn not_found(interpreter: &mut Interpreter, _host: &mut dyn Host, _spec: SpecId) {
    interpreter.instruction_result = InstructionResult::OpcodeNotFound;
}
