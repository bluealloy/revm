use crate::{
    gas, interpreter::Interpreter, primitives::Spec, primitives::SpecId::*, Host, InstructionResult,
};

pub fn chainid<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    // EIP-1344: ChainID opcode
    check!(interpreter, SPEC::enabled(ISTANBUL));
    gas!(interpreter, gas::BASE);
    push!(interpreter, host.env().cfg.chain_id);
}

pub fn coinbase(interpreter: &mut Interpreter, host: &mut dyn Host) {
    gas!(interpreter, gas::BASE);
    push_b256!(interpreter, host.env().block.coinbase.into_word());
}

pub fn timestamp(interpreter: &mut Interpreter, host: &mut dyn Host) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, host.env().block.timestamp);
}

pub fn number(interpreter: &mut Interpreter, host: &mut dyn Host) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, host.env().block.number);
}

pub fn difficulty<H: Host, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    gas!(interpreter, gas::BASE);
    if SPEC::enabled(MERGE) {
        push_b256!(interpreter, host.env().block.prevrandao.unwrap());
    } else {
        push!(interpreter, host.env().block.difficulty);
    }
}

pub fn gaslimit(interpreter: &mut Interpreter, host: &mut dyn Host) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, host.env().block.gas_limit);
}

pub fn gasprice(interpreter: &mut Interpreter, host: &mut dyn Host) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, host.env().effective_gas_price());
}

pub fn basefee<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    gas!(interpreter, gas::BASE);
    // EIP-3198: BASEFEE opcode
    check!(interpreter, SPEC::enabled(LONDON));
    push!(interpreter, host.env().block.basefee);
}

pub fn origin(interpreter: &mut Interpreter, host: &mut dyn Host) {
    gas!(interpreter, gas::BASE);
    push_b256!(interpreter, host.env().tx.caller.into_word());
}
