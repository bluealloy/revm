use crate::{
    gas,
    instructions::InstructionReturn,
    interpreter_types::{Jumps, LoopControl, MemoryTr, RuntimeFlag, StackTr},
    InstructionContextTr, InstructionResult, InterpreterAction,
};
use primitives::{Bytes, U256};

/// Implements the JUMP instruction.
///
/// Unconditional jump to a valid destination.
#[inline]
pub fn jump<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    gas!(context, gas::MID);
    popn!([target], context);
    jump_inner(context, target)
}

/// Implements the JUMPI instruction.
///
/// Conditional jump to a valid destination if condition is true.
#[inline]
pub fn jumpi<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
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
fn jump_inner<C: InstructionContextTr>(context: &mut C, target: U256) -> InstructionReturn {
    let target = as_usize_or_fail_ret!(
        context,
        target,
        InstructionResult::InvalidJump,
        InstructionReturn::halt()
    );
    if !context.bytecode().is_valid_legacy_jump(target) {
        return context.halt(InstructionResult::InvalidJump);
    }
    // SAFETY: `is_valid_jump` ensures that `dest` is in bounds.
    context.bytecode().absolute_jump(target);
    InstructionReturn::cont()
}

/// Implements the JUMPDEST instruction.
///
/// Marks a valid destination for jump operations.
#[inline]
pub fn jumpdest<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    gas!(context, gas::JUMPDEST);
    InstructionReturn::cont()
}

/// Implements the PC instruction.
///
/// Pushes the current program counter onto the stack.
#[inline]
pub fn pc<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    gas!(context, gas::BASE);
    // - 1 because we have already advanced the instruction pointer in `Interpreter::step`
    push!(context, U256::from(context.bytecode().pc() - 1));
    InstructionReturn::cont()
}

/// Internal helper function for return operations.
///
/// Handles memory data retrieval and sets the return action.
#[inline]
fn return_inner<C: InstructionContextTr>(
    context: &mut C,
    instruction_result: InstructionResult,
) -> InstructionReturn {
    popn!([offset, len], context);
    let len = as_usize_or_fail!(context, len);
    // Important: Offset must be ignored if len is zeros
    let mut output = Bytes::default();
    if len != 0 {
        let offset = as_usize_or_fail!(context, offset);
        resize_memory!(context, offset, len);
        output = context.memory().slice_len(offset, len).to_vec().into()
    }

    let action = InterpreterAction::new_return(instruction_result, output, *context.gas());
    context.bytecode().set_action(action);
    InstructionReturn::halt()
}

/// Implements the RETURN instruction.
///
/// Halts execution and returns data from memory.
#[inline]
pub fn ret<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    return_inner(context, InstructionResult::Return)
}

/// EIP-140: REVERT instruction
#[inline]
pub fn revert<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    check!(context, BYZANTIUM);
    return_inner(context, InstructionResult::Revert)
}

/// Stop opcode. This opcode halts the execution.
#[inline]
pub fn stop<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    context.halt(InstructionResult::Stop)
}

/// Invalid opcode. This opcode halts the execution.
#[inline]
pub fn invalid<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    context.halt(InstructionResult::InvalidFEOpcode)
}

/// Unknown opcode. This opcode halts the execution.
#[inline]
pub fn unknown<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    context.halt(InstructionResult::OpcodeNotFound)
}
