use crate::{
    gas,
    instructions::utility::cast_slice_to_u256,
    interpreter_types::{Immediates, InterpreterTypes, Jumps, RuntimeFlag, StackTr},
    InstructionResult,
};
use primitives::U256;

use crate::InstructionContext;

pub fn pop<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    gas!(context.interpreter, gas::BASE);
    // Can ignore return. as relative N jump is safe operation.
    popn!([_i], context.interpreter);
}

/// EIP-3855: PUSH0 instruction
///
/// Introduce a new instruction which pushes the constant value 0 onto the stack.
pub fn push0<WIRE: InterpreterTypes, H: ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    check!(context.interpreter, SHANGHAI);
    gas!(context.interpreter, gas::BASE);
    push!(context.interpreter, U256::ZERO);
}

pub fn push<const N: usize, WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    gas!(context.interpreter, gas::VERYLOW);
    push!(context.interpreter, U256::ZERO);
    popn_top!([], top, context.interpreter);

    let imm = context.interpreter.bytecode.read_slice(N);
    cast_slice_to_u256(imm, top);

    // Can ignore return. as relative N jump is safe operation
    context.interpreter.bytecode.relative_jump(N as isize);
}

pub fn dup<const N: usize, WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    gas!(context.interpreter, gas::VERYLOW);
    if !context.interpreter.stack.dup(N) {
        context.interpreter.halt(InstructionResult::StackOverflow);
    }
}

pub fn swap<const N: usize, WIRE: InterpreterTypes, H: ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    gas!(context.interpreter, gas::VERYLOW);
    assert!(N != 0);
    if !context.interpreter.stack.exchange(0, N) {
        context.interpreter.halt(InstructionResult::StackOverflow);
    }
}
