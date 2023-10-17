use super::env::TxType;
use crate::{Address, Env, TransactTo, U256};
use once_cell::sync::Lazy;
use std::str::FromStr;

const ANCHOR_SELECTOR: u32 = 0xda69d3db;
const ANCHOR_GAS_LIMIT: u64 = 180_000;
static GOLDEN_TOUCH_ACCOUNT: Lazy<Address> = Lazy::new(|| {
    Address::from_str("0x0000777735367b36bC9B61C50022d9D0700dB4Ec")
        .expect("invalid golden touch account")
});

pub static TREASURY: Lazy<Address> = Lazy::new(|| {
    Address::from_str("0xdf09A0afD09a63fb04ab3573922437e1e637dE8b")
        .expect("invalid treasury account")
});

pub(crate) fn validate(env: &Env) -> bool {
    !env.is_anchor()
        || (env.tx.tx_type == TxType::Eip1559
            && env.tx.transact_to == TransactTo::Call(env.taiko.l2_address)
            && u32::from_be_bytes(env.tx.data[..4].try_into().unwrap()) == ANCHOR_SELECTOR
            && env.tx.value == U256::ZERO
            && env.tx.gas_limit == ANCHOR_GAS_LIMIT
            && env.tx.gas_price == env.block.basefee
            && env.tx.caller == *GOLDEN_TOUCH_ACCOUNT)
}
