use crate::{B160, B256, U256};
use sha3::{Digest, Keccak256};

#[inline(always)]
pub fn keccak256(input: &[u8]) -> B256 {
    B256::from_slice(Keccak256::digest(input).as_slice())
}

/// Returns the address for the legacy `CREATE` scheme: [`CreateScheme::Create`]
pub fn create_address(caller: B160, nonce: u64) -> B160 {
    let mut stream = rlp::RlpStream::new_list(2);
    stream.append(&caller.0.as_ref());
    stream.append(&nonce);
    let out = keccak256(&stream.out());
    B160(out[12..].try_into().unwrap())
}

/// Returns the address for the `CREATE2` scheme: [`CreateScheme::Create2`]
pub fn create2_address(caller: B160, code_hash: B256, salt: U256) -> B160 {
    let mut hasher = Keccak256::new();
    hasher.update([0xff]);
    hasher.update(&caller[..]);
    hasher.update(salt.to_be_bytes::<{ U256::BYTES }>());
    hasher.update(&code_hash[..]);

    B160(hasher.finalize().as_slice()[12..].try_into().unwrap())
}
