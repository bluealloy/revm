use crate::{
    interpreter_types::{Immediates, InterpreterTypes, Jumps, StackTr},
    InstructionContext, InstructionResult, STACK_LIMIT,
};
use primitives::U256;

/// Fused PUSH1 + ADD.
pub fn push1_add<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    if context.interpreter.stack.len() == STACK_LIMIT {
        context.interpreter.halt_overflow();
        return;
    }
    let imm = U256::from(context.interpreter.bytecode.read_u8());
    popn_top!([], op, context.interpreter);
    *op = imm.wrapping_add(*op);
    context.interpreter.bytecode.relative_jump(2);
}

/// Fused PUSH1 + SUB.
pub fn push1_sub<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    if context.interpreter.stack.len() == STACK_LIMIT {
        context.interpreter.halt_overflow();
        return;
    }
    let imm = U256::from(context.interpreter.bytecode.read_u8());
    popn_top!([], op, context.interpreter);
    *op = imm.wrapping_sub(*op);
    context.interpreter.bytecode.relative_jump(2);
}

/// Fused PUSH1 + MUL.
pub fn push1_mul<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    if context.interpreter.stack.len() == STACK_LIMIT {
        context.interpreter.halt_overflow();
        return;
    }
    let imm = U256::from(context.interpreter.bytecode.read_u8());
    popn_top!([], op, context.interpreter);
    *op = imm.wrapping_mul(*op);
    context.interpreter.bytecode.relative_jump(2);
}

/// Fused PUSH1 + JUMP.
pub fn push1_jump<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    if context.interpreter.stack.len() == STACK_LIMIT {
        context.interpreter.halt_overflow();
        return;
    }
    let target = U256::from(context.interpreter.bytecode.read_u8());
    let target = as_usize_saturated!(target);
    if !context.interpreter.bytecode.is_valid_legacy_jump(target) {
        context.interpreter.bytecode.relative_jump(2);
        context.interpreter.halt(InstructionResult::InvalidJump);
        return;
    }
    context.interpreter.bytecode.absolute_jump(target);
}

/// Fused PUSH1 + JUMPI.
pub fn push1_jumpi<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    if context.interpreter.stack.len() == STACK_LIMIT {
        context.interpreter.halt_overflow();
        return;
    }
    let target = U256::from(context.interpreter.bytecode.read_u8());
    popn_top!([], cond, context.interpreter);
    if !cond.is_zero() {
        let target = as_usize_saturated!(target);
        if !context.interpreter.bytecode.is_valid_legacy_jump(target) {
            context.interpreter.bytecode.relative_jump(2);
            context.interpreter.halt(InstructionResult::InvalidJump);
            return;
        }
        context.interpreter.bytecode.absolute_jump(target);
    } else {
        context.interpreter.bytecode.relative_jump(2);
    }
}
