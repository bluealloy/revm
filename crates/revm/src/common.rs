use ruint::aliases::{B256, U256};
use sha3::{Digest, Keccak256};

#[inline(always)]
pub fn keccak256(input: &[u8]) -> B256 {
    // TODO(shekhirin): replace with `B256::try_from_be_slice`
    U256::try_from_be_slice(Keccak256::digest(input).as_slice())
        .unwrap()
        .into()
}
