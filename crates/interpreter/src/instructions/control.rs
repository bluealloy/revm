use crate::{
    interpreter::Interpreter,
    interpreter_types::{
        InterpreterTypes as IT, Jumps, LoopControl, MemoryTr, RuntimeFlag, StackTr,
    },
    InstructionExecResult as Result, InstructionResult, InterpreterAction,
};
use context_interface::{cfg::GasParams, Host};
use primitives::{hints_util::cold_path, Bytes, U256};

use crate::InstructionContext as Icx;

/// Implements the JUMP instruction.
///
/// Unconditional jump to a valid destination.
pub fn jump<ITy: IT, H: ?Sized>(context: Icx<'_, H, ITy>) -> Result {
    popn!([target], context.interpreter);
    jump_inner(context.interpreter, target)
}

/// Implements the JUMPI instruction.
///
/// Conditional jump to a valid destination if condition is true.
pub fn jumpi<WIRE: IT, H: ?Sized>(context: Icx<'_, H, WIRE>) -> Result {
    popn!([target, cond], context.interpreter);
    if !cond.is_zero() {
        jump_inner(context.interpreter, target)?;
    }
    Ok(())
}

/// Internal helper function for jump operations.
///
/// Validates jump target and performs the actual jump.
#[inline(always)]
fn jump_inner<WIRE: IT>(
    interpreter: &mut Interpreter<WIRE>,
    target: U256,
) -> Result<(), InstructionResult> {
    let target = as_usize_saturated!(target);
    if !interpreter.bytecode.is_valid_legacy_jump(target) {
        cold_path();
        return Err(InstructionResult::InvalidJump);
    }
    // SAFETY: `is_valid_jump` ensures that `dest` is in bounds.
    interpreter.bytecode.absolute_jump(target);
    Ok(())
}

/// Implements the JUMPDEST instruction.
///
/// Marks a valid destination for jump operations.
pub const fn jumpdest<WIRE: IT, H: ?Sized>(_context: Icx<'_, H, WIRE>) -> Result {
    Ok(())
}

/// Implements the PC instruction.
///
/// Pushes the current program counter onto the stack.
pub fn pc<WIRE: IT, H: ?Sized>(context: Icx<'_, H, WIRE>) -> Result {
    // - 1 because we have already advanced the instruction pointer in `Interpreter::step`
    push!(
        context.interpreter,
        U256::from(context.interpreter.bytecode.pc() - 1)
    );
    Ok(())
}

/// Internal helper function for return operations.
///
/// Handles memory data retrieval and sets the return action.
#[inline]
fn return_inner(
    interpreter: &mut Interpreter<impl IT>,
    gas_params: &GasParams,
    instruction_result: InstructionResult,
) -> Result<(), InstructionResult> {
    popn!([offset, len], interpreter);
    let len = as_usize_or_fail!(interpreter, len);
    // Important: Offset must be ignored if len is zeros
    let mut output = Bytes::default();
    if len != 0 {
        let offset = as_usize_or_fail!(interpreter, offset);
        interpreter.resize_memory(gas_params, offset, len)?;
        output = interpreter.memory.slice_len(offset, len).to_vec().into()
    }

    interpreter
        .bytecode
        .set_action(InterpreterAction::new_return(
            instruction_result,
            output,
            interpreter.gas,
        ));
    Err(instruction_result)
}

/// Implements the RETURN instruction.
///
/// Halts execution and returns data from memory.
pub fn ret<WIRE: IT, H: Host + ?Sized>(context: Icx<'_, H, WIRE>) -> Result {
    return_inner(
        context.interpreter,
        context.host.gas_params(),
        InstructionResult::Return,
    )
}

/// EIP-140: REVERT instruction
pub fn revert<WIRE: IT, H: Host + ?Sized>(context: Icx<'_, H, WIRE>) -> Result {
    check!(context.interpreter, BYZANTIUM);
    return_inner(
        context.interpreter,
        context.host.gas_params(),
        InstructionResult::Revert,
    )
}

/// Stop opcode. This opcode halts the execution.
pub const fn stop<WIRE: IT, H: ?Sized>(_context: Icx<'_, H, WIRE>) -> Result {
    Err(InstructionResult::Stop)
}

/// Invalid opcode. This opcode halts the execution.
pub const fn invalid<WIRE: IT, H: ?Sized>(_context: Icx<'_, H, WIRE>) -> Result {
    Err(InstructionResult::InvalidFEOpcode)
}

/// Unknown opcode. This opcode halts the execution.
pub const fn unknown<WIRE: IT, H: ?Sized>(_context: Icx<'_, H, WIRE>) -> Result {
    Err(InstructionResult::OpcodeNotFound)
}
