use crate::{b256, Address, B256, U256};

pub use alloy_primitives::keccak256;

/// The Keccak-256 hash of the empty string `""`.
pub const KECCAK_EMPTY: B256 =
    b256!("c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470");

/// Returns the address for the legacy `CREATE` scheme: [`CreateScheme::Create`]
#[inline]
pub fn create_address(caller: Address, nonce: u64) -> Address {
    caller.create(nonce)
}

/// Returns the address for the `CREATE2` scheme: [`CreateScheme::Create2`]
#[inline]
pub fn create2_address(caller: Address, code_hash: B256, salt: U256) -> Address {
    caller.create2(salt.to_be_bytes::<32>(), code_hash)
}
