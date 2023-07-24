use super::prelude::*;

// EIP-1344: ChainID opcode
pub(super) fn chainid(interpreter: &mut Interpreter, host: &mut dyn Host, spec: SpecId) {
    check!(interpreter, SpecId::enabled(spec, ISTANBUL));
    gas!(interpreter, gas::BASE);
    push!(interpreter, host.env().cfg.chain_id);
}

pub(super) fn coinbase(interpreter: &mut Interpreter, host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::BASE);
    push_b256!(interpreter, host.env().block.coinbase.into());
}

pub(super) fn timestamp(interpreter: &mut Interpreter, host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, host.env().block.timestamp);
}

pub(super) fn number(interpreter: &mut Interpreter, host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, host.env().block.number);
}

pub(super) fn difficulty(interpreter: &mut Interpreter, host: &mut dyn Host, spec: SpecId) {
    gas!(interpreter, gas::BASE);
    if SpecId::enabled(spec, MERGE) {
        push_b256!(interpreter, host.env().block.prevrandao.unwrap());
    } else {
        push!(interpreter, host.env().block.difficulty);
    }
}

pub(super) fn gaslimit(interpreter: &mut Interpreter, host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, host.env().block.gas_limit);
}

pub(super) fn gasprice(interpreter: &mut Interpreter, host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, host.env().effective_gas_price());
}

// EIP-3198: BASEFEE opcode
pub(super) fn basefee(interpreter: &mut Interpreter, host: &mut dyn Host, spec: SpecId) {
    check!(interpreter, SpecId::enabled(spec, LONDON));
    gas!(interpreter, gas::BASE);
    push!(interpreter, host.env().block.basefee);
}

pub(super) fn origin(interpreter: &mut Interpreter, host: &mut dyn Host, _spec: SpecId) {
    gas!(interpreter, gas::BASE);
    push_b256!(interpreter, host.env().tx.caller.into());
}
