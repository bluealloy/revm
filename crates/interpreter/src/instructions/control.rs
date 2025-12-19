use crate::{
    interpreter::Interpreter,
    interpreter_types::{InterpreterTypes, Jumps, LoopControl, MemoryTr, RuntimeFlag, StackTr},
    InstructionResult, InterpreterAction,
};
use primitives::{Bytes, U256};

use crate::InstructionContext;

/// Implements the JUMP instruction.
///
/// Unconditional jump to a valid destination.
pub fn jump<EXT, ITy: InterpreterTypes<Extend = EXT>, H: ?Sized>(context: InstructionContext<'_, EXT, H, ITy>) {
    popn!([target], context.interpreter);
    jump_inner(context.interpreter, target);
}

/// Implements the JUMPI instruction.
///
/// Conditional jump to a valid destination if condition is true.
pub fn jumpi<EXT, WIRE: InterpreterTypes<Extend = EXT>, H: ?Sized>(context: InstructionContext<'_, EXT, H, WIRE>) {
    popn!([target, cond], context.interpreter);
    if !cond.is_zero() {
        jump_inner(context.interpreter, target);
    }
}

/// Internal helper function for jump operations.
///
/// Validates jump target and performs the actual jump.
#[inline(always)]
fn jump_inner<EXT, WIRE: InterpreterTypes<Extend = EXT>>(interpreter: &mut Interpreter<EXT, WIRE>, target: U256) {
    let target = as_usize_or_fail!(interpreter, target, InstructionResult::InvalidJump);
    if !interpreter.bytecode.is_valid_legacy_jump(target) {
        interpreter.halt(InstructionResult::InvalidJump);
        return;
    }
    // SAFETY: `is_valid_jump` ensures that `dest` is in bounds.
    interpreter.bytecode.absolute_jump(target);
}

/// Implements the JUMPDEST instruction.
///
/// Marks a valid destination for jump operations.
pub fn jumpdest<EXT, WIRE: InterpreterTypes<Extend = EXT>, H: ?Sized>(_context: InstructionContext<'_, EXT, H, WIRE>) {}

/// Implements the PC instruction.
///
/// Pushes the current program counter onto the stack.
pub fn pc<EXT, WIRE: InterpreterTypes<Extend = EXT>, H: ?Sized>(context: InstructionContext<'_, EXT, H, WIRE>) {
    // - 1 because we have already advanced the instruction pointer in `Interpreter::step`
    push!(
        context.interpreter,
        U256::from(context.interpreter.bytecode.pc() - 1)
    );
}

#[inline]
/// Internal helper function for return operations.
///
/// Handles memory data retrieval and sets the return action.
fn return_inner<EXT>(
    interpreter: &mut Interpreter<EXT, impl InterpreterTypes<Extend = EXT>>,
    instruction_result: InstructionResult,
) {
    popn!([offset, len], interpreter);
    let len = as_usize_or_fail!(interpreter, len);
    // Important: Offset must be ignored if len is zeros
    let mut output = Bytes::default();
    if len != 0 {
        let offset = as_usize_or_fail!(interpreter, offset);
        if !interpreter.resize_memory(offset, len) {
            return;
        }
        output = interpreter.memory.slice_len(offset, len).to_vec().into()
    }

    interpreter
        .bytecode
        .set_action(InterpreterAction::new_return(
            instruction_result,
            output,
            interpreter.gas,
        ));
}

/// Implements the RETURN instruction.
///
/// Halts execution and returns data from memory.
pub fn ret<EXT, WIRE: InterpreterTypes<Extend = EXT>, H: ?Sized>(context: InstructionContext<'_, EXT, H, WIRE>) {
    return_inner(context.interpreter, InstructionResult::Return);
}

/// EIP-140: REVERT instruction
pub fn revert<EXT, WIRE: InterpreterTypes<Extend = EXT>, H: ?Sized>(context: InstructionContext<'_, EXT, H, WIRE>) {
    check!(context.interpreter, BYZANTIUM);
    return_inner(context.interpreter, InstructionResult::Revert);
}

/// Stop opcode. This opcode halts the execution.
pub fn stop<EXT, WIRE: InterpreterTypes<Extend = EXT>, H: ?Sized>(context: InstructionContext<'_, EXT, H, WIRE>) {
    context.interpreter.halt(InstructionResult::Stop);
}

/// Invalid opcode. This opcode halts the execution.
pub fn invalid<EXT, WIRE: InterpreterTypes<Extend = EXT>, H: ?Sized>(context: InstructionContext<'_, EXT, H, WIRE>) {
    context.interpreter.halt(InstructionResult::InvalidFEOpcode);
}

/// Unknown opcode. This opcode halts the execution.
pub fn unknown<EXT, WIRE: InterpreterTypes<Extend = EXT>, H: ?Sized>(context: InstructionContext<'_, EXT, H, WIRE>) {
    context.interpreter.halt(InstructionResult::OpcodeNotFound);
}
