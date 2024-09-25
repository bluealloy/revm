use crate::{gas, Host, Interpreter};
use primitives::U256;
use specification::hardfork::Spec;
use wiring::Transaction;

pub fn gasprice<H: Host + ?Sized>(interpreter: &mut Interpreter, host: &mut H) {
    gas!(interpreter, gas::BASE);
    push!(interpreter, host.env().effective_gas_price());
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
