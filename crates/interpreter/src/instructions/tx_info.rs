use crate::{
    interpreter_types::{InterpreterTypes, RuntimeFlag, StackTr},
    Host,
};

use crate::InstructionContext;

/// Implements the GASPRICE instruction.
///
/// Gets the gas price of the originating transaction.
pub fn gasprice<WIRE: InterpreterTypes, H: Host + ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    push!(context.interpreter, context.host.effective_gas_price());
}

/// Implements the ORIGIN instruction.
///
/// Gets the execution origination address.
pub fn origin<WIRE: InterpreterTypes, H: Host + ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    push!(
        context.interpreter,
        context.host.caller().into_word().into()
    );
}

/// Implements the BLOBHASH instruction.
///
/// EIP-4844: Shard Blob Transactions - gets the hash of a transaction blob.
pub fn blob_hash<WIRE: InterpreterTypes, H: Host + ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    check!(context.interpreter, CANCUN);
    popn_top!([], index, context.interpreter);
    let i = as_usize_saturated!(index);
    *index = context.host.blob_hash(i).unwrap_or_default();
}
