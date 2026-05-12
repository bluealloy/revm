use crate::{
    interpreter_types::{InterpreterTypes as ITy, RuntimeFlag, StackTr},
    Host, InstructionContext as Ictx, InstructionExecResult as Result,
};

/// Implements the GASPRICE instruction.
///
/// Gets the gas price of the originating transaction.
pub fn gasprice<IT: ITy, H: Host + ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    push!(context.interpreter, context.host.effective_gas_price());
    Ok(())
}

/// Implements the ORIGIN instruction.
///
/// Gets the execution origination address.
pub fn origin<IT: ITy, H: Host + ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    push!(
        context.interpreter,
        context.host.caller().into_word().into()
    );
    Ok(())
}

/// Implements the BLOBHASH instruction.
///
/// EIP-4844: Shard Blob Transactions - gets the hash of a transaction blob.
pub fn blob_hash<IT: ITy, H: Host + ?Sized>(context: Ictx<'_, H, IT>) -> Result {
    check!(context.interpreter, CANCUN);
    popn_top!([], index, context.interpreter);
    let i = as_usize_saturated!(*index);
    *index = context.host.blob_hash(i).unwrap_or_default();
    Ok(())
}
