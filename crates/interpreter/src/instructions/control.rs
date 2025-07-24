use crate::{
    gas,
    instructions::InstructionReturn,
    interpreter_types::{InterpreterTypes, Jumps, LoopControl, MemoryTr, RuntimeFlag, StackTr},
    InstructionResult, InterpreterAction,
};
use primitives::{Bytes, U256};

use crate::InstructionContext;

/// Implements the JUMP instruction.
///
/// Unconditional jump to a valid destination.
pub fn jump<ITy: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, ITy>,
) -> InstructionReturn {
    gas!(context, gas::MID);
    popn!([target], context);
    jump_inner(context, target)
}

/// Implements the JUMPI instruction.
///
/// Conditional jump to a valid destination if condition is true.
pub fn jumpi<WIRE: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    gas!(context, gas::HIGH);
    popn!([target, cond], context);

    if !cond.is_zero() {
        jump_inner(context, target)
    } else {
        InstructionReturn::cont()
    }
}

/// Internal helper function for jump operations.
///
/// Validates jump target and performs the actual jump.
#[inline(always)]
#[allow(clippy::unused_unit)]
fn jump_inner<WIRE: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
    target: U256,
) -> InstructionReturn {
    let target = as_usize_or_fail_ret!(
        context,
        target,
        InstructionResult::InvalidJump,
        InstructionReturn::halt()
    );
    if !context.interpreter.bytecode.is_valid_legacy_jump(target) {
        context.halt(InstructionResult::InvalidJump);
        return InstructionReturn::halt();
    }
    // SAFETY: `is_valid_jump` ensures that `dest` is in bounds.
    context.absolute_jump(target);
    InstructionReturn::cont()
}

/// Implements the JUMPDEST instruction.
///
/// Marks a valid destination for jump operations.
pub fn jumpdest<WIRE: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    gas!(context, gas::JUMPDEST);
    InstructionReturn::cont()
}

/// Implements the PC instruction.
///
/// Pushes the current program counter onto the stack.
pub fn pc<WIRE: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    gas!(context, gas::BASE);
    // - 1 because we have already advanced the instruction pointer in `Interpreter::step`
    push!(context, U256::from(context.pc() - 1));
    InstructionReturn::cont()
}

/// Internal helper function for return operations.
///
/// Handles memory data retrieval and sets the return action.
#[inline]
#[allow(clippy::unused_unit)]
fn return_inner<WIRE: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
    instruction_result: InstructionResult,
) {
    // Zero gas cost
    // gas!(interpreter, gas::ZERO)
    popn!([offset, len], context, ());
    let len = as_usize_or_fail_ret!(context, len, ());
    // Important: Offset must be ignored if len is zeros
    let mut output = Bytes::default();
    if len != 0 {
        let offset = as_usize_or_fail_ret!(context, offset, ());
        resize_memory!(context, offset, len, ());
        output = context
            .interpreter
            .memory
            .slice_len(offset, len)
            .to_vec()
            .into()
    }

    context
        .interpreter
        .bytecode
        .set_action(InterpreterAction::new_return(
            instruction_result,
            output,
            context.interpreter.gas,
        ));
}

/// Implements the RETURN instruction.
///
/// Halts execution and returns data from memory.
pub fn ret<WIRE: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    return_inner(context, InstructionResult::Return);
    InstructionReturn::halt()
}

/// EIP-140: REVERT instruction
pub fn revert<WIRE: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    check!(context, BYZANTIUM);
    return_inner(context, InstructionResult::Revert);
    InstructionReturn::halt()
}

/// Stop opcode. This opcode halts the execution.
pub fn stop<WIRE: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    context.halt(InstructionResult::Stop);
    InstructionReturn::halt()
}

/// Invalid opcode. This opcode halts the execution.
pub fn invalid<WIRE: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    context.halt(InstructionResult::InvalidFEOpcode);
    InstructionReturn::halt()
}

/// Unknown opcode. This opcode halts the execution.
pub fn unknown<WIRE: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    context.halt(InstructionResult::OpcodeNotFound);
    InstructionReturn::halt()
}
