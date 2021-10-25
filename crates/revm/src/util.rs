use primitive_types::{H160, H256};
use sha3::{Digest, Keccak256};

pub fn l64(gas: u64) -> u64 {
    gas - gas / 64
}

pub fn create_address(caller: H160, nonce: u64) -> H160 {
    let mut stream = rlp::RlpStream::new_list(2);
    stream.append(&caller);
    stream.append(&nonce);
    H256::from_slice(Keccak256::digest(&stream.out()).as_slice()).into()
}

/// Get the create address from given scheme.
pub fn create2_address(caller: H160, code_hash: H256, salt: H256) -> H160 {
    let mut hasher = Keccak256::new();
    hasher.update(&[0xff]);
    hasher.update(&caller[..]);
    hasher.update(&salt[..]);
    hasher.update(&code_hash[..]);
    H256::from_slice(hasher.finalize().as_slice()).into()
}
