use crate::bits::B256;
use sha3::{Digest, Keccak256};

#[inline(always)]
pub fn keccak256(input: &[u8]) -> B256 {
    B256::from_slice(Keccak256::digest(input).as_slice())
}
