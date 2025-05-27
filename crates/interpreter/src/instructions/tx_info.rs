use crate::{
    gas,
    interpreter_types::{InterpreterTypes, LoopControl, RuntimeFlag, StackTr},
    Host,
};
use primitives::U256;

use super::context::InstructionContext;

pub fn gasprice<WIRE: InterpreterTypes, H: Host + ?Sized>(
    ctx: &mut InstructionContext<'_, H, WIRE>,
) {
    gas!(ctx.interpreter, gas::BASE);
    push!(ctx.interpreter, U256::from(ctx.host.effective_gas_price()));
}

pub fn origin<WIRE: InterpreterTypes, H: Host + ?Sized>(ctx: &mut InstructionContext<'_, H, WIRE>) {
    gas!(ctx.interpreter, gas::BASE);
    push!(ctx.interpreter, ctx.host.caller().into_word().into());
}

// EIP-4844: Shard Blob Transactions
pub fn blob_hash<WIRE: InterpreterTypes, H: Host + ?Sized>(
    ctx: &mut InstructionContext<'_, H, WIRE>,
) {
    check!(ctx.interpreter, CANCUN);
    gas!(ctx.interpreter, gas::VERYLOW);
    popn_top!([], index, ctx.interpreter);
    let i = as_usize_saturated!(index);
    *index = ctx.host.blob_hash(i).unwrap_or_default();
}
