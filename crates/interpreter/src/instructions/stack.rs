use crate::{
    gas,
    instructions::InstructionReturn,
    interpreter_types::{Immediates, Jumps, RuntimeFlag, StackTr},
    InstructionContextTr, InstructionResult,
};
use primitives::U256;

/// Implements the POP instruction.
///
/// Removes the top item from the stack.
#[inline]
pub fn pop<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    gas!(context, gas::BASE);
    // Can ignore return. as relative N jump is safe operation.
    popn!([_i], context);
    InstructionReturn::cont()
}

/// EIP-3855: PUSH0 instruction
///
/// Introduce a new instruction which pushes the constant value 0 onto the stack.
#[inline]
pub fn push0<C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    check!(context, SHANGHAI);
    gas!(context, gas::BASE);
    push!(context, U256::ZERO);
    InstructionReturn::cont()
}

/// Implements the PUSH1-PUSH32 instructions.
///
/// Pushes N bytes from bytecode onto the stack as a 32-byte value.
#[inline]
pub fn push<const N: usize, C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    gas!(context, gas::VERYLOW);

    let slice = fuck_lt!(context.bytecode().read_slice(N));
    if !context.stack().push_slice(slice) {
        return context.halt(InstructionResult::StackOverflow);
    }

    // Can ignore return. as relative N jump is safe operation
    context.bytecode().relative_jump(N as isize);
    InstructionReturn::cont()
}

/// Implements the DUP1-DUP16 instructions.
///
/// Duplicates the Nth stack item to the top of the stack.
#[inline]
pub fn dup<const N: usize, C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    gas!(context, gas::VERYLOW);
    if !context.stack().dup(N) {
        return context.halt(InstructionResult::StackOverflow);
    }
    InstructionReturn::cont()
}

/// Implements the SWAP1-SWAP16 instructions.
///
/// Swaps the top stack item with the Nth stack item.
#[inline]
pub fn swap<const N: usize, C: InstructionContextTr>(context: &mut C) -> InstructionReturn {
    gas!(context, gas::VERYLOW);
    assert!(N != 0);
    if !context.stack().exchange(0, N) {
        return context.halt(InstructionResult::StackOverflow);
    }
    InstructionReturn::cont()
}
