use revm_primitives::Bytes;

use crate::{
    gas,
    primitives::{Spec, U256},
    Host, InstructionResult, Interpreter, InterpreterResult,
};

pub fn jump<H: Host>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::MID);
    pop!(interpreter, dest);
    let dest = as_usize_or_fail!(interpreter, dest, InstructionResult::InvalidJump);
    if interpreter.contract.is_valid_jump(dest) {
        // SAFETY: In analysis we are checking create our jump table and we do check above to be
        // sure that jump is safe to execute.
        interpreter.instruction_pointer =
            unsafe { interpreter.contract.bytecode.as_ptr().add(dest) };
    } else {
        interpreter.instruction_result = InstructionResult::InvalidJump;
    }
}

pub fn jumpi<H: Host>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::HIGH);
    pop!(interpreter, dest, value);
    if value != U256::ZERO {
        let dest = as_usize_or_fail!(interpreter, dest, InstructionResult::InvalidJump);
        if interpreter.contract.is_valid_jump(dest) {
            // SAFETY: In analysis we are checking if jump is valid destination and
            // this `if` makes this unsafe block safe.
            interpreter.instruction_pointer =
                unsafe { interpreter.contract.bytecode.as_ptr().add(dest) };
        } else {
            interpreter.instruction_result = InstructionResult::InvalidJump
        }
    }
}

pub fn jumpdest<H: Host>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::JUMPDEST);
}

pub fn pc<H: Host>(interpreter: &mut Interpreter, _host: &mut H) {
    gas!(interpreter, gas::BASE);
    // - 1 because we have already advanced the instruction pointer in `Interpreter::step`
    push!(interpreter, U256::from(interpreter.program_counter() - 1));
}

#[inline(always)]
fn return_inner(interpreter: &mut Interpreter, instruction_result: InstructionResult) {
    // zero gas cost
    // gas!(interpreter, gas::ZERO);
    pop!(interpreter, offset, len);
    let len = as_usize_or_fail!(interpreter, len);
    // important: offset must be ignored if len is zeros
    let mut output = Bytes::default();
    if len != 0 {
        let offset = as_usize_or_fail!(interpreter, offset);
        shared_memory_resize!(interpreter, offset, len);

        output = interpreter.shared_memory.slice(offset, len).to_vec().into()
    }
    interpreter.instruction_result = instruction_result;
    interpreter.next_action = Some(crate::InterpreterAction::Return {
        result: InterpreterResult {
            output,
            gas: interpreter.gas,
            result: instruction_result,
        },
    });
}

pub fn ret<H: Host>(interpreter: &mut Interpreter, _host: &mut H) {
    return_inner(interpreter, InstructionResult::Return);
}

/// EIP-140: REVERT instruction
pub fn revert<H: Host, SPEC: Spec>(interpreter: &mut Interpreter, _host: &mut H) {
    check!(interpreter, BYZANTIUM);
    return_inner(interpreter, InstructionResult::Revert);
}

/// Stop opcode. This opcode halts the execution.
pub fn stop<H: Host>(interpreter: &mut Interpreter, _host: &mut H) {
    interpreter.instruction_result = InstructionResult::Stop;
}

/// Invalid opcode. This opcode halts the execution.
pub fn invalid<H: Host>(interpreter: &mut Interpreter, _host: &mut H) {
    interpreter.instruction_result = InstructionResult::InvalidFEOpcode;
}

/// Unknown opcode. This opcode halts the execution.
pub fn unknown<H: Host>(interpreter: &mut Interpreter, _host: &mut H) {
    interpreter.instruction_result = InstructionResult::OpcodeNotFound;
}
