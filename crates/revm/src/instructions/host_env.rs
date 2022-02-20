use crate::{interpreter::Interpreter, Host, Return, Spec, SpecId::*};
use primitive_types::H256;

pub fn chainid<H: Host, SPEC: Spec>(machine: &mut Interpreter, host: &mut H) -> Return {
    // gas!(machine, gas::BASE);
    // EIP-1344: ChainID opcode
    check!(SPEC::enabled(ISTANBUL));
    push!(machine, host.env().cfg.chain_id);
    Return::Continue
}

pub fn coinbase<H: Host>(machine: &mut Interpreter, host: &mut H) -> Return {
    // gas!(machine, gas::BASE);
    push_h256!(machine, host.env().block.coinbase.into());
    Return::Continue
}

pub fn timestamp<H: Host>(machine: &mut Interpreter, host: &mut H) -> Return {
    // gas!(machine, gas::BASE);
    push!(machine, host.env().block.timestamp);
    Return::Continue
}

pub fn number<H: Host>(machine: &mut Interpreter, host: &mut H) -> Return {
    // gas!(machine, gas::BASE);
    push!(machine, host.env().block.number);
    Return::Continue
}

pub fn difficulty<H: Host>(machine: &mut Interpreter, host: &mut H) -> Return {
    // gas!(machine, gas::BASE);
    push!(machine, host.env().block.difficulty);
    Return::Continue
}

pub fn gaslimit<H: Host>(machine: &mut Interpreter, host: &mut H) -> Return {
    // gas!(machine, gas::BASE);
    push!(machine, host.env().block.gas_limit);
    Return::Continue
}

pub fn gasprice<H: Host>(machine: &mut Interpreter, host: &mut H) -> Return {
    // gas!(machine, gas::BASE);
    push!(machine, host.env().effective_gas_price());
    Return::Continue
}

pub fn basefee<H: Host, SPEC: Spec>(machine: &mut Interpreter, host: &mut H) -> Return {
    // gas!(machine, gas::BASE);
    // EIP-3198: BASEFEE opcode
    check!(SPEC::enabled(LONDON));
    push!(machine, host.env().block.basefee);
    Return::Continue
}

pub fn origin<H: Host>(machine: &mut Interpreter, host: &mut H) -> Return {
    // gas!(machine, gas::BASE);
    let ret = H256::from(host.env().tx.caller);
    push_h256!(machine, ret);
    Return::Continue
}
