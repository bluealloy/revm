use crate::{gas, Host, Interpreter};
use primitives::U256;
use specification::hardfork::Spec;
use transaction::Eip4844Tx;
use wiring::{Block, Transaction, TransactionType};

pub fn gasprice<H: Host + ?Sized>(interpreter: &mut Interpreter, host: &mut H) {
    gas!(interpreter, gas::BASE);
    let env = host.env();
    let basefee = *env.block.basefee();
    push!(interpreter, env.tx.effective_gas_price(basefee));
}

pub fn origin<H: Host + ?Sized>(interpreter: &mut Interpreter, host: &mut H) {
    gas!(interpreter, gas::BASE);
    push_b256!(
        interpreter,
        host.env().tx.common_fields().caller().into_word()
    );
}

// EIP-4844: Shard Blob Transactions
pub fn blob_hash<H: Host + ?Sized, SPEC: Spec>(interpreter: &mut Interpreter, host: &mut H) {
    check!(interpreter, CANCUN);
    gas!(interpreter, gas::VERYLOW);
    pop_top!(interpreter, index);
    let i = as_usize_saturated!(index);
    let tx = &host.env().tx;
    *index = if tx.tx_type().into() == TransactionType::Eip4844 {
        tx.eip4844()
            .blob_versioned_hashes()
            .get(i)
            .cloned()
            .map(|b| U256::from_be_bytes(*b))
            .unwrap_or(U256::ZERO)
    } else {
        U256::ZERO
    };
}
