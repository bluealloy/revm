use primitive_types::H256;
use sha3::{Digest, Keccak256};

#[inline(always)]
pub fn keccak256(input: &[u8]) -> H256 {
    H256::from_slice(Keccak256::digest(input).as_ref())
}
