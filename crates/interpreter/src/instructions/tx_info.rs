use crate::{
    interpreter_types::{InterpreterTypes as IT, RuntimeFlag, StackTr},
    Host, InstructionContext as Icx, InstructionExecResult as Result,
};

/// Implements the GASPRICE instruction.
///
/// Gets the gas price of the originating transaction.
pub fn gasprice<WIRE: IT, H: Host + ?Sized>(context: Icx<'_, H, WIRE>) -> Result {
    push!(context.interpreter, context.host.effective_gas_price());
    Ok(())
}

/// Implements the ORIGIN instruction.
///
/// Gets the execution origination address.
pub fn origin<WIRE: IT, H: Host + ?Sized>(context: Icx<'_, H, WIRE>) -> Result {
    push!(
        context.interpreter,
        context.host.caller().into_word().into()
    );
    Ok(())
}

/// Implements the BLOBHASH instruction.
///
/// EIP-4844: Shard Blob Transactions - gets the hash of a transaction blob.
pub fn blob_hash<WIRE: IT, H: Host + ?Sized>(context: Icx<'_, H, WIRE>) -> Result {
    check!(context.interpreter, CANCUN);
    popn_top!([], index, context.interpreter);
    let i = as_usize_saturated!(*index);
    *index = context.host.blob_hash(i).unwrap_or_default();
    Ok(())
}
