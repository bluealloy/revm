use crate::{gas, interpreter::InterpreterTrait, Host, Interpreter};
use primitives::U256;
use specification::hardfork::{Spec, SpecId::*};
use wiring::Block;

/// EIP-1344: ChainID opcode
pub fn chainid<I: InterpreterTrait, H: Host + ?Sized>(interpreter: &mut I, host: &mut H) {
    check!(interpreter, ISTANBUL);
    gas!(interpreter, gas::BASE);
    push!(interpreter, U256::from(host.env().cfg.chain_id));
}

pub fn coinbase<I: InterpreterTrait, H: Host + ?Sized>(interpreter: &mut I, host: &mut H) {
    gas!(interpreter, gas::BASE);
    push_b256!(interpreter, host.env().block.beneficiary().into_word());
}

pub fn timestamp<I: InterpreterTrait, H: Host + ?Sized>(interpreter: &mut I, host: &mut H) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, *host.env().block.timestamp());
}

pub fn block_number<I: InterpreterTrait, H: Host + ?Sized>(interpreter: &mut I, host: &mut H) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, *host.env().block.number());
}

pub fn difficulty<I: InterpreterTrait, H: Host + ?Sized>(interpreter: &mut I, host: &mut H) {
    gas!(interpreter, gas::BASE);
    if interpreter.spec_id().is_enabled_in(MERGE) {
        push_b256!(interpreter, *host.env().block.prevrandao().unwrap());
    } else {
        push!(interpreter, *host.env().block.difficulty());
    }
}

pub fn gaslimit<I: InterpreterTrait, H: Host + ?Sized>(interpreter: &mut I, host: &mut H) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, *host.env().block.gas_limit());
}

/// EIP-3198: BASEFEE opcode
pub fn basefee<I: InterpreterTrait, H: Host + ?Sized>(interpreter: &mut I, host: &mut H) {
    check!(interpreter, LONDON);
    gas!(interpreter, gas::BASE);
    push!(interpreter, *host.env().block.basefee());
}

/// EIP-7516: BLOBBASEFEE opcode
pub fn blob_basefee<I: InterpreterTrait, H: Host + ?Sized>(interpreter: &mut I, host: &mut H) {
    check!(interpreter, CANCUN);
    gas!(interpreter, gas::BASE);
    push!(
        interpreter,
        U256::from(host.env().block.blob_gasprice().unwrap_or_default())
    );
}
