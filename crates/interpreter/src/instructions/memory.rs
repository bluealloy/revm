use crate::{
    gas,
    instructions::InstructionReturn,
    interpreter_types::{InterpreterTypes, MemoryTr, RuntimeFlag, StackTr},
};
use core::cmp::max;
use primitives::U256;

use crate::InstructionContext;

/// Implements the MLOAD instruction.
///
/// Loads a 32-byte word from memory.
pub fn mload<WIRE: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    gas!(context, gas::VERYLOW);
    popn_top!([], top, context);
    let offset = as_usize_or_fail!(context, top);
    resize_memory!(context, offset, 32);
    *top =
        U256::try_from_be_slice(context.interpreter.memory.slice_len(offset, 32).as_ref()).unwrap();
    InstructionReturn::cont()
}

/// Implements the MSTORE instruction.
///
/// Stores a 32-byte word to memory.
pub fn mstore<WIRE: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    gas!(context, gas::VERYLOW);
    popn!([offset, value], context);
    let offset = as_usize_or_fail!(context, offset);
    resize_memory!(context, offset, 32);
    context
        .interpreter
        .memory
        .set(offset, &value.to_be_bytes::<32>());
    InstructionReturn::cont()
}

/// Implements the MSTORE8 instruction.
///
/// Stores a single byte to memory.
pub fn mstore8<WIRE: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    gas!(context, gas::VERYLOW);
    popn!([offset, value], context);
    let offset = as_usize_or_fail!(context, offset);
    resize_memory!(context, offset, 1);
    context.interpreter.memory.set(offset, &[value.byte(0)]);
    InstructionReturn::cont()
}

/// Implements the MSIZE instruction.
///
/// Gets the size of active memory in bytes.
pub fn msize<WIRE: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    gas!(context, gas::BASE);
    push!(
        context,
        U256::from(context.interpreter.memory.size())
    );
    InstructionReturn::cont()
}

/// Implements the MCOPY instruction.
///
/// EIP-5656: Memory copying instruction that copies memory from one location to another.
pub fn mcopy<WIRE: InterpreterTypes, H: ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    check!(context, CANCUN);
    popn!([dst, src, len], context);

    // Into usize or fail
    let len = as_usize_or_fail!(context, len);
    // Deduce gas
    gas_or_fail!(context, gas::copy_cost_verylow(len));
    if len == 0 {
        return InstructionReturn::cont();
    }

    let dst = as_usize_or_fail!(context, dst);
    let src = as_usize_or_fail!(context, src);
    // Resize memory
    resize_memory!(context, max(dst, src), len);
    // Copy memory in place
    context.interpreter.memory.copy(dst, src, len);
    InstructionReturn::cont()
}
