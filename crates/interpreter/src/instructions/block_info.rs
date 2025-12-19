use crate::{
    interpreter_types::{InterpreterTypes, RuntimeFlag, StackTr},
    Host,
};
use primitives::hardfork::SpecId::*;

use crate::InstructionContext;

/// EIP-1344: ChainID opcode
pub fn chainid<EXT, WIRE: InterpreterTypes<Extend = EXT>, H: Host + ?Sized>(context: InstructionContext<'_, EXT, H, WIRE>) {
    check!(context.interpreter, ISTANBUL);
    push!(context.interpreter, context.host.chain_id());
}

/// Implements the COINBASE instruction.
///
/// Pushes the current block's beneficiary address onto the stack.
pub fn coinbase<EXT, WIRE: InterpreterTypes<Extend = EXT>, H: Host + ?Sized>(
    context: InstructionContext<'_, EXT, H, WIRE>,
) {
    push!(
        context.interpreter,
        context.host.beneficiary().into_word().into()
    );
}

/// Implements the TIMESTAMP instruction.
///
/// Pushes the current block's timestamp onto the stack.
pub fn timestamp<EXT, WIRE: InterpreterTypes<Extend = EXT>, H: Host + ?Sized>(
    context: InstructionContext<'_, EXT, H, WIRE>,
) {
    push!(context.interpreter, context.host.timestamp());
}

/// Implements the NUMBER instruction.
///
/// Pushes the current block number onto the stack.
pub fn block_number<EXT, WIRE: InterpreterTypes<Extend = EXT>, H: Host + ?Sized>(
    context: InstructionContext<'_, EXT, H, WIRE>,
) {
    push!(context.interpreter, context.host.block_number());
}

/// Implements the DIFFICULTY/PREVRANDAO instruction.
///
/// Pushes the block difficulty (pre-merge) or prevrandao (post-merge) onto the stack.
pub fn difficulty<EXT, WIRE: InterpreterTypes<Extend = EXT>, H: Host + ?Sized>(
    context: InstructionContext<'_, EXT, H, WIRE>,
) {
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
pub fn gaslimit<EXT, WIRE: InterpreterTypes<Extend = EXT>, H: Host + ?Sized>(
    context: InstructionContext<'_, EXT, H, WIRE>,
) {
    push!(context.interpreter, context.host.gas_limit());
}

/// EIP-3198: BASEFEE opcode
pub fn basefee<EXT, WIRE: InterpreterTypes<Extend = EXT>, H: Host + ?Sized>(context: InstructionContext<'_, EXT, H, WIRE>) {
    check!(context.interpreter, LONDON);
    push!(context.interpreter, context.host.basefee());
}

/// EIP-7516: BLOBBASEFEE opcode
pub fn blob_basefee<EXT, WIRE: InterpreterTypes<Extend = EXT>, H: Host + ?Sized>(
    context: InstructionContext<'_, EXT, H, WIRE>,
) {
    check!(context.interpreter, CANCUN);
    push!(context.interpreter, context.host.blob_gasprice());
}
