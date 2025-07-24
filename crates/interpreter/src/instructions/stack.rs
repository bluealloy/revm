use crate::{
    gas,
    instructions::InstructionReturn,
    interpreter_types::{InterpreterTypes, RuntimeFlag, StackTr},
    InstructionContext, InstructionResult,
};
use primitives::U256;

/// Implements the POP instruction.
///
/// Removes the top item from the stack.
#[inline]
pub fn pop<WIRE: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    gas!(context, gas::BASE);
    // Can ignore return. as relative N jump is safe operation.
    popn!([_i], context);
    InstructionReturn::cont()
}

/// EIP-3855: PUSH0 instruction
///
/// Introduce a new instruction which pushes the constant value 0 onto the stack.
#[inline]
pub fn push0<WIRE: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    check!(context, SHANGHAI);
    gas!(context, gas::BASE);
    push!(context, U256::ZERO);
    InstructionReturn::cont()
}

/// Implements the PUSH1-PUSH32 instructions.
///
/// Pushes N bytes from bytecode onto the stack as a 32-byte value.
#[inline]
pub fn push<const N: usize, WIRE: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    gas!(context, gas::VERYLOW);

    let slice = context.read_slice(N);
    if !context.interpreter.stack.push_slice(slice) {
        context.halt(InstructionResult::StackOverflow);
        return InstructionReturn::halt();
    }

    // Can ignore return. as relative N jump is safe operation
    context.relative_jump(N as isize);
    InstructionReturn::cont()
}

/// Implements the DUP1-DUP16 instructions.
///
/// Duplicates the Nth stack item to the top of the stack.
#[inline]
pub fn dup<const N: usize, WIRE: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    gas!(context, gas::VERYLOW);
    if !context.interpreter.stack.dup(N) {
        context.halt(InstructionResult::StackOverflow);
    }
    InstructionReturn::cont()
}

/// Implements the SWAP1-SWAP16 instructions.
///
/// Swaps the top stack item with the Nth stack item.
#[inline]
pub fn swap<const N: usize, WIRE: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    gas!(context, gas::VERYLOW);
    assert!(N != 0);
    if !context.interpreter.stack.exchange(0, N) {
        context.halt(InstructionResult::StackOverflow);
    }
    InstructionReturn::cont()
}
