
use tiny_keccak::{Hasher, Keccak};
use primitive_types::H256;

#[inline(always)]
pub fn keccak256(input: &[u8]) -> H256 {
    let mut tiny_sha3 = Keccak::v256();
    tiny_sha3.update(input);

    let mut out = H256::zero();
    tiny_sha3.finalize(&mut out.0);
    out
}