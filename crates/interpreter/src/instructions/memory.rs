use crate::{
    gas,
    interpreter_types::{InterpreterTypes, LoopControl, MemoryTr, RuntimeFlag, StackTr},
    Host,
};
use core::cmp::max;
use primitives::U256;

use super::context::InstructionContext;

pub fn mload<WIRE: InterpreterTypes, H: Host + ?Sized>(ctx: &mut InstructionContext<'_, H, WIRE>) {
    gas!(ctx.interpreter, gas::VERYLOW);
    popn_top!([], top, ctx.interpreter);
    let offset = as_usize_or_fail!(ctx.interpreter, top);
    resize_memory!(ctx.interpreter, offset, 32);
    *top = U256::try_from_be_slice(ctx.interpreter.memory.slice_len(offset, 32).as_ref()).unwrap()
}

pub fn mstore<WIRE: InterpreterTypes, H: Host + ?Sized>(ctx: &mut InstructionContext<'_, H, WIRE>) {
    gas!(ctx.interpreter, gas::VERYLOW);
    popn!([offset, value], ctx.interpreter);
    let offset = as_usize_or_fail!(ctx.interpreter, offset);
    resize_memory!(ctx.interpreter, offset, 32);
    ctx.interpreter
        .memory
        .set(offset, &value.to_be_bytes::<32>());
}

pub fn mstore8<WIRE: InterpreterTypes, H: Host + ?Sized>(
    ctx: &mut InstructionContext<'_, H, WIRE>,
) {
    gas!(ctx.interpreter, gas::VERYLOW);
    popn!([offset, value], ctx.interpreter);
    let offset = as_usize_or_fail!(ctx.interpreter, offset);
    resize_memory!(ctx.interpreter, offset, 1);
    ctx.interpreter.memory.set(offset, &[value.byte(0)]);
}

pub fn msize<WIRE: InterpreterTypes, H: Host + ?Sized>(ctx: &mut InstructionContext<'_, H, WIRE>) {
    gas!(ctx.interpreter, gas::BASE);
    push!(ctx.interpreter, U256::from(ctx.interpreter.memory.size()));
}

// EIP-5656: MCOPY - Memory copying instruction
pub fn mcopy<WIRE: InterpreterTypes, H: Host + ?Sized>(ctx: &mut InstructionContext<'_, H, WIRE>) {
    check!(ctx.interpreter, CANCUN);
    popn!([dst, src, len], ctx.interpreter);

    // Into usize or fail
    let len = as_usize_or_fail!(ctx.interpreter, len);
    // Deduce gas
    gas_or_fail!(ctx.interpreter, gas::copy_cost_verylow(len));
    if len == 0 {
        return;
    }

    let dst = as_usize_or_fail!(ctx.interpreter, dst);
    let src = as_usize_or_fail!(ctx.interpreter, src);
    // Resize memory
    resize_memory!(ctx.interpreter, max(dst, src), len);
    // Copy memory in place
    ctx.interpreter.memory.copy(dst, src, len);
}
