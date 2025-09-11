use crate::{
    interpreter_types::{InterpreterTypes, RuntimeFlag, StackTr},
    Host,
};
use primitives::U256;

use crate::InstructionContext;

/// Implements the GASPRICE instruction.
///
/// Gets the gas price of the originating transaction.
pub fn gasprice<WIRE: InterpreterTypes, H: Host + ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) -> bool {
    //gas!(context.interpreter, gas::BASE);
    push!(
        context.interpreter,
        U256::from(context.host.effective_gas_price())
    );
    true
}

/// Implements the ORIGIN instruction.
///
/// Gets the execution origination address.
pub fn origin<WIRE: InterpreterTypes, H: Host + ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) -> bool {
    //gas!(context.interpreter, gas::BASE);
    push!(
        context.interpreter,
        context.host.caller().into_word().into()
    );
    true
}

/// Implements the BLOBHASH instruction.
///
/// EIP-4844: Shard Blob Transactions - gets the hash of a transaction blob.
pub fn blob_hash<WIRE: InterpreterTypes, H: Host + ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) -> bool {
    check!(context.interpreter, CANCUN);
    //gas!(context.interpreter, gas::VERYLOW);
    popn_top!([], index, context.interpreter);
    let i = as_usize_saturated!(index);
    *index = context.host.blob_hash(i).unwrap_or_default();
    true
}
