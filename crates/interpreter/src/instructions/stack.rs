use crate::{
    interpreter_types::{Immediates, InterpreterTypes, Jumps, RuntimeFlag, StackTr},
    InstructionResult,
};
use primitives::U256;

use crate::InstructionContext;

/// Implements the POP instruction.
///
/// Removes the top item from the stack.
pub fn pop<EXT, WIRE: InterpreterTypes<Extend = EXT>, H: ?Sized>(context: InstructionContext<'_, EXT, H, WIRE>) {
    // Can ignore return. as relative N jump is safe operation.
    popn!([_i], context.interpreter);
}

/// EIP-3855: PUSH0 instruction
///
/// Introduce a new instruction which pushes the constant value 0 onto the stack.
pub fn push0<EXT, WIRE: InterpreterTypes<Extend = EXT>, H: ?Sized>(context: InstructionContext<'_, EXT, H, WIRE>) {
    check!(context.interpreter, SHANGHAI);
    push!(context.interpreter, U256::ZERO);
}

/// Implements the PUSH1-PUSH32 instructions.
///
/// Pushes N bytes from bytecode onto the stack as a 32-byte value.
pub fn push<const N: usize, EXT, WIRE: InterpreterTypes<Extend = EXT>, H: ?Sized>(
    context: InstructionContext<'_, EXT, H, WIRE>,
) {
    let slice = context.interpreter.bytecode.read_slice(N);
    if !context.interpreter.stack.push_slice(slice) {
        context.interpreter.halt(InstructionResult::StackOverflow);
        return;
    }

    // Can ignore return. as relative N jump is safe operation
    context.interpreter.bytecode.relative_jump(N as isize);
}

/// Implements the DUP1-DUP16 instructions.
///
/// Duplicates the Nth stack item to the top of the stack.
pub fn dup<const N: usize, EXT, WIRE: InterpreterTypes<Extend = EXT>, H: ?Sized>(
    context: InstructionContext<'_, EXT, H, WIRE>,
) {
    if !context.interpreter.stack.dup(N) {
        context.interpreter.halt(InstructionResult::StackOverflow);
    }
}

/// Implements the SWAP1-SWAP16 instructions.
///
/// Swaps the top stack item with the Nth stack item.
pub fn swap<const N: usize, EXT, WIRE: InterpreterTypes<Extend = EXT>, H: ?Sized>(
    context: InstructionContext<'_, EXT, H, WIRE>,
) {
    assert!(N != 0);
    if !context.interpreter.stack.exchange(0, N) {
        context.interpreter.halt(InstructionResult::StackOverflow);
    }
}
