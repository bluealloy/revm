use crate::{interpreter::Interpreter, Host, Return, Spec, SpecId::*};
use primitive_types::H256;

pub fn chainid<H: Host, SPEC: Spec>(interp: &mut Interpreter, host: &mut H) -> Return {
    // gas!(interp, gas::BASE);
    // EIP-1344: ChainID opcode
    check!(SPEC::enabled(ISTANBUL));
    push!(interp, host.env().cfg.chain_id);
    Return::Continue
}

pub fn coinbase<H: Host>(interp: &mut Interpreter, host: &mut H) -> Return {
    // gas!(interp, gas::BASE);
    push_h256!(interp, host.env().block.coinbase.into());
    Return::Continue
}

pub fn timestamp<H: Host>(interp: &mut Interpreter, host: &mut H) -> Return {
    // gas!(interp, gas::BASE);
    push!(interp, host.env().block.timestamp);
    Return::Continue
}

pub fn number<H: Host>(interp: &mut Interpreter, host: &mut H) -> Return {
    // gas!(interp, gas::BASE);
    push!(interp, host.env().block.number);
    Return::Continue
}

pub fn difficulty<H: Host, SPEC: Spec>(interp: &mut Interpreter, host: &mut H) -> Return {
    // gas!(interp, gas::BASE);
    if SPEC::enabled(MERGE) {
        push_h256!(interp, host.env().block.prevrandao.unwrap());
    } else {
        push!(interp, host.env().block.difficulty);
    }
    Return::Continue
}

pub fn gaslimit<H: Host>(interp: &mut Interpreter, host: &mut H) -> Return {
    // gas!(interp, gas::BASE);
    push!(interp, host.env().block.gas_limit);
    Return::Continue
}

pub fn gasprice<H: Host>(interp: &mut Interpreter, host: &mut H) -> Return {
    // gas!(interp, gas::BASE);
    push!(interp, host.env().effective_gas_price());
    Return::Continue
}

pub fn basefee<H: Host, SPEC: Spec>(interp: &mut Interpreter, host: &mut H) -> Return {
    // gas!(interp, gas::BASE);
    // EIP-3198: BASEFEE opcode
    check!(SPEC::enabled(LONDON));
    push!(interp, host.env().block.basefee);
    Return::Continue
}

pub fn origin<H: Host>(interp: &mut Interpreter, host: &mut H) -> Return {
    // gas!(interp, gas::BASE);
    let ret = H256::from(host.env().tx.caller);
    push_h256!(interp, ret);
    Return::Continue
}
