use crate::{
    interpreter_types::{InterpreterTypes, RuntimeFlag, StackTr},
    Host,
};
use primitives::{hardfork::SpecId::*, U256};

use crate::InstructionContext;

/// EIP-1344: ChainID opcode
pub fn chainid<WIRE: InterpreterTypes, H: Host + ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    check!(context.interpreter, ISTANBUL);
    //gas!(context.interpreter, gas::BASE);
    push!(context.interpreter, context.host.chain_id());
}

/// Implements the COINBASE instruction.
///
/// Pushes the current block's beneficiary address onto the stack.
pub fn coinbase<WIRE: InterpreterTypes, H: Host + ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    //gas!(context.interpreter, gas::BASE);
    push!(
        context.interpreter,
        context.host.beneficiary().into_word().into()
    );
}

/// Implements the TIMESTAMP instruction.
///
/// Pushes the current block's timestamp onto the stack.
pub fn timestamp<WIRE: InterpreterTypes, H: Host + ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    //gas!(context.interpreter, gas::BASE);
    push!(context.interpreter, context.host.timestamp());
}

/// Implements the NUMBER instruction.
///
/// Pushes the current block number onto the stack.
pub fn block_number<WIRE: InterpreterTypes, H: Host + ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    //gas!(context.interpreter, gas::BASE);
    push!(context.interpreter, U256::from(context.host.block_number()));
}

/// Implements the DIFFICULTY/PREVRANDAO instruction.
///
/// Pushes the block difficulty (pre-merge) or prevrandao (post-merge) onto the stack.
pub fn difficulty<WIRE: InterpreterTypes, H: Host + ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    //gas!(context.interpreter, gas::BASE);
    if context
        .interpreter
        .runtime_flag
        .spec_id()
        .is_enabled_in(MERGE)
    {
        // Unwrap is safe as this fields is checked in validation handler.
        push!(context.interpreter, context.host.prevrandao().unwrap());
    } else {
        push!(context.interpreter, context.host.difficulty());
    }
}

/// Implements the GASLIMIT instruction.
///
/// Pushes the current block's gas limit onto the stack.
pub fn gaslimit<WIRE: InterpreterTypes, H: Host + ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    //gas!(context.interpreter, gas::BASE);
    push!(context.interpreter, context.host.gas_limit());
}

/// EIP-3198: BASEFEE opcode
pub fn basefee<WIRE: InterpreterTypes, H: Host + ?Sized>(context: InstructionContext<'_, H, WIRE>) {
    check!(context.interpreter, LONDON);
    //gas!(context.interpreter, gas::BASE);
    push!(context.interpreter, context.host.basefee());
}

/// EIP-7516: BLOBBASEFEE opcode
pub fn blob_basefee<WIRE: InterpreterTypes, H: Host + ?Sized>(
    context: InstructionContext<'_, H, WIRE>,
) {
    check!(context.interpreter, CANCUN);
    //gas!(context.interpreter, gas::BASE);
    push!(context.interpreter, context.host.blob_gasprice());
}
