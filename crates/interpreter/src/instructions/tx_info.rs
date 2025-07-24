use crate::{
    gas,
    instructions::InstructionReturn,
    interpreter_types::{InterpreterTypes, RuntimeFlag, StackTr},
    Host,
};
use primitives::U256;

use crate::InstructionContext;

/// Implements the GASPRICE instruction.
///
/// Gets the gas price of the originating transaction.
#[inline]
pub fn gasprice<WIRE: InterpreterTypes, H: Host + ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    gas!(context, gas::BASE);
    push!(
        context,
        U256::from(context.host.effective_gas_price())
    );
    InstructionReturn::cont()
}

/// Implements the ORIGIN instruction.
///
/// Gets the execution origination address.
#[inline]
pub fn origin<WIRE: InterpreterTypes, H: Host + ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    gas!(context, gas::BASE);
    push!(
        context,
        context.host.caller().into_word().into()
    );
    InstructionReturn::cont()
}

/// Implements the BLOBHASH instruction.
///
/// EIP-4844: Shard Blob Transactions - gets the hash of a transaction blob.
#[inline]
pub fn blob_hash<WIRE: InterpreterTypes, H: Host + ?Sized>(
    context: &mut InstructionContext<'_, H, WIRE>,
) -> InstructionReturn {
    check!(context, CANCUN);
    gas!(context, gas::VERYLOW);
    popn_top!([], index, context);
    let i = as_usize_saturated!(index);
    *index = context.host.blob_hash(i).unwrap_or_default();
    InstructionReturn::cont()
}
