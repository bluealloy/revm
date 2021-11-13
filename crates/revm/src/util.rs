use primitive_types::{H160, H256, U256};
use sha3::{Digest, Keccak256};

pub fn l64(gas: u64) -> u64 {
    gas - gas / 64
}

pub fn create_address(caller: H160, nonce: u64) -> H160 {
    let mut stream = rlp::RlpStream::new_list(2);
    stream.append(&caller);
    stream.append(&nonce);
    let out = H256::from_slice(Keccak256::digest(&stream.out()).as_slice());
    let out = H160::from_slice(&out.as_bytes()[12..]);
    out
}

/// Get the create address from given scheme.
pub fn create2_address(caller: H160, code_hash: H256, salt: U256) -> H160 {
    let mut temp: [u8; 32] = [0; 32];
    salt.to_big_endian(&mut temp);

    let mut hasher = Keccak256::new();
    hasher.update(&[0xff]);
    hasher.update(&caller[..]);
    hasher.update(&temp);
    hasher.update(&code_hash[..]);
    H160::from_slice(&hasher.finalize().as_slice()[12..])
}
