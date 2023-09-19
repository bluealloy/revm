use crate::{
    gas,
    primitives::{Spec, U256},
    Host, InstructionResult, Interpreter,
};

pub fn jump(interpreter: &mut Interpreter, _host: &mut dyn Host) {
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

pub fn jumpi(interpreter: &mut Interpreter, _host: &mut dyn Host) {
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

pub fn jumpdest(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::JUMPDEST);
}

pub fn pc(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    gas!(interpreter, gas::BASE);
    // - 1 because we have already advanced the instruction pointer in `Interpreter::step`
    push!(interpreter, U256::from(interpreter.program_counter() - 1));
}

#[inline]
pub fn ret(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    // zero gas cost
    // gas!(interpreter, gas::ZERO);
    pop!(interpreter, offset, len);
    let len = as_usize_or_fail!(interpreter, len);
    // important: offset must be ignored if len is zero
    if len != 0 {
        let offset = as_usize_or_fail!(interpreter, offset);
        memory_resize!(interpreter, offset, len);
        interpreter.return_offset = offset;
    }
    interpreter.return_len = len;
    interpreter.instruction_result = InstructionResult::Return;
}

/// EIP-140: REVERT instruction
pub fn revert<SPEC: Spec>(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    // zero gas cost
    // gas!(interpreter, gas::ZERO);
    check!(interpreter, BYZANTIUM);
    ret(interpreter, _host);
    interpreter.instruction_result = InstructionResult::Revert;
}

pub fn stop(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    interpreter.instruction_result = InstructionResult::Stop;
}

pub fn invalid(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    interpreter.instruction_result = InstructionResult::InvalidFEOpcode;
}

pub fn not_found(interpreter: &mut Interpreter, _host: &mut dyn Host) {
    interpreter.instruction_result = InstructionResult::OpcodeNotFound;
}
