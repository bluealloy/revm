use crate::{B160, B256, TARGET_BLOB_GAS_PER_BLOCK, U256};
use hex_literal::hex;
use sha3::{Digest, Keccak256};

pub const KECCAK_EMPTY: B256 = B256(hex!(
    "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"
));

#[inline(always)]
pub fn keccak256(input: &[u8]) -> B256 {
    B256(Keccak256::digest(input)[..].try_into().unwrap())
}

/// Returns the address for the legacy `CREATE` scheme: [`crate::env::CreateScheme::Create`]
pub fn create_address(caller: B160, nonce: u64) -> B160 {
    let mut stream = rlp::RlpStream::new_list(2);
    stream.append(&caller.0.as_ref());
    stream.append(&nonce);
    let out = keccak256(&stream.out());
    B160(out[12..].try_into().unwrap())
}

/// Returns the address for the `CREATE2` scheme: [`crate::env::CreateScheme::Create2`]
pub fn create2_address(caller: B160, code_hash: B256, salt: U256) -> B160 {
    let mut hasher = Keccak256::new();
    hasher.update([0xff]);
    hasher.update(&caller[..]);
    hasher.update(salt.to_be_bytes::<{ U256::BYTES }>());
    hasher.update(&code_hash[..]);

    B160(hasher.finalize().as_slice()[12..].try_into().unwrap())
}

/// Calculates the [EIP-4844] `excess_blob_gas` from the parent header's `blob_gas_used` and
/// `excess_blob_gas`.
///
/// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
#[inline]
pub fn calc_excess_blob_gas(parent_blob_gas_used: u64, parent_excess_blob_gas: u64) -> u64 {
    let excess = parent_blob_gas_used.saturating_add(parent_excess_blob_gas);
    TARGET_BLOB_GAS_PER_BLOCK.saturating_sub(excess)
}

/// Approximates `factor * e ** (numerator / denominator)` using Taylor expansion.
#[inline]
pub fn fake_exponential(factor: u64, numerator: u64, denominator: u64) -> u64 {
    assert!(denominator > 0, "attempt to divide by zero");

    let mut i = 1;
    let mut output = 0;
    let mut numerator_accum = factor * denominator;
    while numerator_accum > 0 {
        output += numerator_accum;
        // SAFETY: asserted > 0 above
        numerator_accum = unsafe {
            (numerator_accum * numerator)
                .checked_div(denominator * i)
                .unwrap_unchecked()
        };
        i += 1;
    }
    output / denominator
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
