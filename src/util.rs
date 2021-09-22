use primitive_types::{H160, H256};
use sha3::{Digest, Keccak256};

use crate::CreateScheme;

macro_rules! try_or_fail {
    ( $e:expr ) => {
        match $e {
            Ok(v) => v,
            Err(e) => return Capture::Exit((e.into(), None, Vec::new())),
        }
    };
}

pub fn l64(gas: u64) -> u64 {
    gas - gas / 64
}

/// Get the create address from given scheme.
pub fn create_address(scheme: CreateScheme) -> H160 {
    match scheme {
        CreateScheme::Create2 {
            caller,
            code_hash,
            salt,
        } => {
            let mut hasher = Keccak256::new();
            hasher.update(&[0xff]);
            hasher.update(&caller[..]);
            hasher.update(&salt[..]);
            hasher.update(&code_hash[..]);
            H256::from_slice(hasher.finalize().as_slice()).into()
        }
        CreateScheme::Legacy { caller, nonce } => {
            let mut stream = rlp::RlpStream::new_list(2);
            stream.append(&caller);
            stream.append(&nonce);
            H256::from_slice(Keccak256::digest(&stream.out()).as_slice()).into()
        }
        CreateScheme::Fixed(naddress) => naddress,
    }
}
