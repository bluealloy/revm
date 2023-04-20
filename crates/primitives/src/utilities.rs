use crate::{B160, B256, U256};
use hex_literal::hex;
use sha3::{Digest, Keccak256};

pub const KECCAK_EMPTY: B256 = B256(hex!(
    "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"
));

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

/// Serde functions to serde as [bytes::Bytes] hex string
#[cfg(feature = "serde")]
pub mod serde_hex_bytes {
    use alloc::string::{String, ToString};
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S, T>(x: T, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AsRef<[u8]>,
    {
        s.serialize_str(&alloc::format!("0x{}", hex::encode(x.as_ref())))
    }

    pub fn deserialize<'de, D>(d: D) -> Result<bytes::Bytes, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(d)?;
        if let Some(value) = value.strip_prefix("0x") {
            hex::decode(value)
        } else {
            hex::decode(&value)
        }
        .map(Into::into)
        .map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}
