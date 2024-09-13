use crate::{
    gas,
    primitives::{Block, Spec, SpecId::*, Transaction, U256},
    Host, Interpreter,
};

/// EIP-1344: ChainID opcode
pub fn chainid<H: Host + ?Sized, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    check!(interpreter, ISTANBUL);
    gas!(interpreter, gas::BASE);
    push!(interpreter, U256::from(host.env().cfg.chain_id));
}

pub fn coinbase<H: Host + ?Sized>(interpreter: &mut Interpreter, host: &mut H) {
    gas!(interpreter, gas::BASE);
    push_b256!(interpreter, host.env().block.coinbase().into_word());
}

pub fn timestamp<H: Host + ?Sized>(interpreter: &mut Interpreter, host: &mut H) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, *host.env().block.timestamp());
}

pub fn block_number<H: Host + ?Sized>(interpreter: &mut Interpreter, host: &mut H) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, *host.env().block.number());
}

pub fn difficulty<H: Host + ?Sized, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    gas!(interpreter, gas::BASE);
    if SPEC::enabled(MERGE) {
        push_b256!(interpreter, *host.env().block.prevrandao().unwrap());
    } else {
        push!(interpreter, *host.env().block.difficulty());
    }
}

pub fn gaslimit<H: Host + ?Sized>(interpreter: &mut Interpreter, host: &mut H) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, *host.env().block.gas_limit());
}

pub fn gasprice<H: Host + ?Sized>(interpreter: &mut Interpreter, host: &mut H) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, host.env().effective_gas_price());
}

/// EIP-3198: BASEFEE opcode
pub fn basefee<H: Host + ?Sized, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    check!(interpreter, LONDON);
    gas!(interpreter, gas::BASE);
    push!(interpreter, *host.env().block.basefee());
}

pub fn origin<H: Host + ?Sized>(interpreter: &mut Interpreter, host: &mut H) {
    gas!(interpreter, gas::BASE);
    push_b256!(interpreter, host.env().tx.caller().into_word());
}

// EIP-4844: Shard Blob Transactions
pub fn blob_hash<H: Host + ?Sized, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    check!(interpreter, CANCUN);
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, index);
    let i = as_usize_saturated!(index);
    *index = match host.env().tx.blob_hashes().get(i) {
        Some(hash) => U256::from_be_bytes(hash.0),
        None => U256::ZERO,
    };
}

/// EIP-7516: BLOBBASEFEE opcode
pub fn blob_basefee<H: Host + ?Sized, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    check!(interpreter, CANCUN);
    gas!(interpreter, gas::BASE);
    push!(
        interpreter,
        U256::from(
            host.env()
                .block
                .get_blob_gasprice()
                .copied()
                .unwrap_or_default()
        )
    );
}
