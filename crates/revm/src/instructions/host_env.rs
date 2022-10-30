use crate::{interpreter::Interpreter, Host, Return, Spec, SpecId::*};
use ruint::aliases::{B256, U256};

pub fn chainid<H: Host, SPEC: Spec>(interp: &mut Interpreter, host: &mut H) -> Return {
    // gas!(interp, gas::BASE);
    // EIP-1344: ChainID opcode
    check!(SPEC::enabled(ISTANBUL));
    push!(interp, host.env().cfg.chain_id);
    Return::Continue
}

pub fn coinbase<H: Host>(interp: &mut Interpreter, host: &mut H) -> Return {
    // gas!(interp, gas::BASE);
    push_b256!(
        interp,
        // TODO(shekhirin): replace with `B256::from(bits: Bits)`
        B256::from(U256::from(host.env().block.coinbase.into_inner()))
    );
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

pub fn difficulty<H: Host>(interp: &mut Interpreter, host: &mut H) -> Return {
    // gas!(interp, gas::BASE);
    push!(interp, host.env().block.difficulty);
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
    push_b256!(
        interp,
        // TODO(shekhirin): replace with `B256::from(bits: Bits)`
        B256::from(U256::from(host.env().tx.caller.into_inner()))
    );
    Return::Continue
}
