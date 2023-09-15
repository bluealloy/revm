use crate::{
    gas,
    interpreter::Interpreter,
    primitives::{Spec, SpecId::*, U256},
    Host, InstructionResult,
};

pub fn chainid<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    // EIP-1344: ChainID opcode
    check!(interpreter, SPEC::enabled(ISTANBUL));
    gas!(interpreter, gas::BASE);
    push!(interpreter, U256::from(host.env().cfg.chain_id));
}

pub fn coinbase(interpreter: &mut Interpreter, host: &mut dyn Host) {
    gas!(interpreter, gas::BASE);
    push_b256!(interpreter, host.env().block.coinbase.into());
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
    push_b256!(interpreter, host.env().tx.caller.into());
}

// EIP-4844: Shard Blob Transactions
pub fn blob_hash<SPEC: Spec>(interpreter: &mut Interpreter, host: &mut dyn Host) {
    check!(interpreter, SPEC::enabled(CANCUN));
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, index);
    let i = as_usize_saturated!(index);
    *index = match host.env().tx.blob_hashes.get(i) {
        Some(hash) => U256::from_be_bytes(hash.0),
        None => U256::ZERO,
    };
}
