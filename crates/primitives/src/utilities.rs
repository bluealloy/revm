use crate::{
    B160, B256, BLOB_GASPRICE_UPDATE_FRACTION, MIN_BLOB_GASPRICE, TARGET_BLOB_GAS_PER_BLOCK, U256,
};
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

/// Calculates the `excess_blob_gas` from the parent header's `blob_gas_used` and `excess_blob_gas`.
///
/// See also [the EIP-4844 helpers](https://eips.ethereum.org/EIPS/eip-4844#helpers).
#[inline]
pub fn calc_excess_blob_gas(parent_excess_blob_gas: u64, parent_blob_gas_used: u64) -> u64 {
    (parent_excess_blob_gas + parent_blob_gas_used).saturating_sub(TARGET_BLOB_GAS_PER_BLOCK)
}

/// Calculates the blob gasprice from the header's excess blob gas field.
///
/// See also [the EIP-4844 helpers](https://eips.ethereum.org/EIPS/eip-4844#helpers).
#[inline]
pub fn calc_blob_gasprice(excess_blob_gas: u64) -> u64 {
    fake_exponential(
        MIN_BLOB_GASPRICE,
        excess_blob_gas,
        BLOB_GASPRICE_UPDATE_FRACTION,
    )
}

/// Approximates `factor * e ** (numerator / denominator)` using Taylor expansion.
///
/// This is used to calculate the blob price.
///
/// See also [the EIP-4844 helpers](https://eips.ethereum.org/EIPS/eip-4844#helpers).
///
/// # Panic
///
/// Panics if `denominator` is zero.
#[inline]
pub fn fake_exponential(factor: u64, numerator: u64, denominator: u64) -> u64 {
    assert_ne!(denominator, 0, "attempt to divide by zero");
    let factor = factor as u128;
    let numerator = numerator as u128;
    let denominator = denominator as u128;

    let mut i = 1;
    let mut output = 0;
    let mut numerator_accum = factor * denominator;
    while numerator_accum > 0 {
        output += numerator_accum;

        // Denominator is asserted as not zero at the start of the function.
        numerator_accum = (numerator_accum * numerator) / (denominator * i);
        i += 1;
    }
    (output / denominator) as u64
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GAS_PER_BLOB;

    // https://github.com/ethereum/go-ethereum/blob/28857080d732857030eda80c69b9ba2c8926f221/consensus/misc/eip4844/eip4844_test.go#L27
    #[test]
    fn test_calc_excess_blob_gas() {
        for t @ &(excess, blobs, expected) in &[
            // The excess blob gas should not increase from zero if the used blob
            // slots are below - or equal - to the target.
            (0, 0, 0),
            (0, 1, 0),
            (0, TARGET_BLOB_GAS_PER_BLOCK / GAS_PER_BLOB, 0),
            // If the target blob gas is exceeded, the excessBlobGas should increase
            // by however much it was overshot
            (
                0,
                (TARGET_BLOB_GAS_PER_BLOCK / GAS_PER_BLOB) + 1,
                GAS_PER_BLOB,
            ),
            (
                1,
                (TARGET_BLOB_GAS_PER_BLOCK / GAS_PER_BLOB) + 1,
                GAS_PER_BLOB + 1,
            ),
            (
                1,
                (TARGET_BLOB_GAS_PER_BLOCK / GAS_PER_BLOB) + 2,
                2 * GAS_PER_BLOB + 1,
            ),
            // The excess blob gas should decrease by however much the target was
            // under-shot, capped at zero.
            (
                TARGET_BLOB_GAS_PER_BLOCK,
                TARGET_BLOB_GAS_PER_BLOCK / GAS_PER_BLOB,
                TARGET_BLOB_GAS_PER_BLOCK,
            ),
            (
                TARGET_BLOB_GAS_PER_BLOCK,
                (TARGET_BLOB_GAS_PER_BLOCK / GAS_PER_BLOB) - 1,
                TARGET_BLOB_GAS_PER_BLOCK - GAS_PER_BLOB,
            ),
            (
                TARGET_BLOB_GAS_PER_BLOCK,
                (TARGET_BLOB_GAS_PER_BLOCK / GAS_PER_BLOB) - 2,
                TARGET_BLOB_GAS_PER_BLOCK - (2 * GAS_PER_BLOB),
            ),
            (
                GAS_PER_BLOB - 1,
                (TARGET_BLOB_GAS_PER_BLOCK / GAS_PER_BLOB) - 1,
                0,
            ),
        ] {
            let actual = calc_excess_blob_gas(excess, blobs * GAS_PER_BLOB);
            assert_eq!(actual, expected, "test: {t:?}");
        }
    }

    // https://github.com/ethereum/go-ethereum/blob/28857080d732857030eda80c69b9ba2c8926f221/consensus/misc/eip4844/eip4844_test.go#L60
    #[test]
    fn test_calc_blob_fee() {
        for &(excess, expected) in &[(0, 1), (2314057, 1), (2314058, 2), (10 * 1024 * 1024, 23)] {
            let actual = calc_blob_gasprice(excess);
            assert_eq!(actual, expected, "test: {excess}");
        }
    }

    // https://github.com/ethereum/go-ethereum/blob/28857080d732857030eda80c69b9ba2c8926f221/consensus/misc/eip4844/eip4844_test.go#L78
    #[test]
    fn fake_exp() {
        for t @ &(factor, numerator, denominator, expected) in &[
            (1u64, 0u64, 1u64, 1u64),
            (38493, 0, 1000, 38493),
            (0, 1234, 2345, 0),
            (1, 2, 1, 6), // approximate 7.389
            (1, 4, 2, 6),
            (1, 3, 1, 16), // approximate 20.09
            (1, 6, 2, 18),
            (1, 4, 1, 49), // approximate 54.60
            (1, 8, 2, 50),
            (10, 8, 2, 542), // approximate 540.598
            (11, 8, 2, 596), // approximate 600.58
            (1, 5, 1, 136),  // approximate 148.4
            (1, 5, 2, 11),   // approximate 12.18
            (2, 5, 2, 23),   // approximate 24.36
            (1, 50000000, 2225652, 5709098764),
            (1, 380928, BLOB_GASPRICE_UPDATE_FRACTION, 1),
        ] {
            let actual = fake_exponential(factor, numerator, denominator);
            assert_eq!(actual, expected, "test: {t:?}");
        }
    }
}
