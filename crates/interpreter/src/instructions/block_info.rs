use crate::{
    gas,
    interpreter_types::{InterpreterTypes, LoopControl, RuntimeFlag, StackTr},
    Host,
};
use primitives::{hardfork::SpecId::*, U256};

use super::context::InstructionContext;

/// EIP-1344: ChainID opcode
pub fn chainid<WIRE: InterpreterTypes, H: Host + ?Sized>(
    ctx: &mut InstructionContext<'_, H, WIRE>,
) {
    check!(ctx.interpreter, ISTANBUL);
    gas!(ctx.interpreter, gas::BASE);
    push!(ctx.interpreter, ctx.host.chain_id());
}

pub fn coinbase<WIRE: InterpreterTypes, H: Host + ?Sized>(
    ctx: &mut InstructionContext<'_, H, WIRE>,
) {
    gas!(ctx.interpreter, gas::BASE);
    push!(ctx.interpreter, ctx.host.beneficiary().into_word().into());
}

pub fn timestamp<WIRE: InterpreterTypes, H: Host + ?Sized>(
    ctx: &mut InstructionContext<'_, H, WIRE>,
) {
    gas!(ctx.interpreter, gas::BASE);
    push!(ctx.interpreter, ctx.host.timestamp());
}

pub fn block_number<WIRE: InterpreterTypes, H: Host + ?Sized>(
    ctx: &mut InstructionContext<'_, H, WIRE>,
) {
    gas!(ctx.interpreter, gas::BASE);
    push!(ctx.interpreter, U256::from(ctx.host.block_number()));
}

pub fn difficulty<WIRE: InterpreterTypes, H: Host + ?Sized>(
    ctx: &mut InstructionContext<'_, H, WIRE>,
) {
    gas!(ctx.interpreter, gas::BASE);
    if ctx.interpreter.runtime_flag.spec_id().is_enabled_in(MERGE) {
        // Unwrap is safe as this fields is checked in validation handler.
        push!(ctx.interpreter, ctx.host.prevrandao().unwrap());
    } else {
        push!(ctx.interpreter, ctx.host.difficulty());
    }
}

pub fn gaslimit<WIRE: InterpreterTypes, H: Host + ?Sized>(
    ctx: &mut InstructionContext<'_, H, WIRE>,
) {
    gas!(ctx.interpreter, gas::BASE);
    push!(ctx.interpreter, ctx.host.gas_limit());
}

/// EIP-3198: BASEFEE opcode
pub fn basefee<WIRE: InterpreterTypes, H: Host + ?Sized>(
    ctx: &mut InstructionContext<'_, H, WIRE>,
) {
    check!(ctx.interpreter, LONDON);
    gas!(ctx.interpreter, gas::BASE);
    push!(ctx.interpreter, ctx.host.basefee());
}

/// EIP-7516: BLOBBASEFEE opcode
pub fn blob_basefee<WIRE: InterpreterTypes, H: Host + ?Sized>(
    ctx: &mut InstructionContext<'_, H, WIRE>,
) {
    check!(ctx.interpreter, CANCUN);
    gas!(ctx.interpreter, gas::BASE);
    push!(ctx.interpreter, ctx.host.blob_gasprice());
}
