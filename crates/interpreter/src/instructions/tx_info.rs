use crate::{
    gas,
    interpreter_types::{InterpreterTypes, LoopControl, RuntimeFlag, StackTr},
    Host,
};
use primitives::U256;

use super::control::InstructionContext;

pub fn gasprice<WIRE: InterpreterTypes, H: Host + ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) {
    gas!(context.interpreter, gas::BASE);
    push!(
        context.interpreter,
        U256::from(context.host.effective_gas_price())
    );
}

pub fn origin<WIRE: InterpreterTypes, H: Host + ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) {
    gas!(context.interpreter, gas::BASE);
    push!(
        context.interpreter,
        context.host.caller().into_word().into()
    );
}

// EIP-4844: Shard Blob Transactions
pub fn blob_hash<WIRE: InterpreterTypes, H: Host + ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) {
    check!(context.interpreter, CANCUN);
    gas!(context.interpreter, gas::VERYLOW);
    popn_top!([], index, context.interpreter);
    let i = as_usize_saturated!(index);
    *index = context.host.blob_hash(i).unwrap_or_default();
}
