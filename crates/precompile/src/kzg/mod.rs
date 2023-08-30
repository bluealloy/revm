use crate::{Error, Precompile, PrecompileAddress, PrecompileResult, B160};
use c_kzg::{bindings, Bytes32, Bytes48, CkzgError, KzgSettings};
use once_cell::sync::OnceCell;
use revm_primitives::hex_literal::hex;
use sha2::{Digest, Sha256};

#[rustfmt::skip]
mod generated_settings;

pub const POINT_EVALUATION: PrecompileAddress =
    PrecompileAddress(ADDRESS, Precompile::Standard(run));

const ADDRESS: B160 = crate::u64_to_b160(0x0A);
const GAS_COST: u64 = 50_000;
const VERSIONED_HASH_VERSION_KZG: u8 = 0x01;

#[allow(dead_code)]
const FIELD_ELEMENTS_PER_BLOB: u64 = 4096;

/// The big-endian representation of the modulus.
#[allow(dead_code)]
const BLS_MODULUS: &[u8; 32] =
    &hex!("73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001");

/// `U256(FIELD_ELEMENTS_PER_BLOB).to_be_bytes() ++ BLS_MODULUS.to_bytes32()`
const RETURN_VALUE: &[u8; 64] = &hex!(
    "0000000000000000000000000000000000000000000000000000000000001000"
    "73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001"
);

fn run(input: &[u8], gas_limit: u64) -> PrecompileResult {
    if gas_limit < GAS_COST {
        return Err(Error::OutOfGas);
    }
    if input.len() != 192 {
        return Err(Error::KzgInvalidInputLength);
    }

    // Verify commitment matches versioned_hash
    let commitment = &input[96..144];
    let versioned_hash = &input[0..32];
    if kzg_to_versioned_hash(commitment) != versioned_hash {
        return Err(Error::KzgInvalidCommitment);
    }

    // Verify KZG proof
    let commitment = as_bytes48(commitment);
    let z = as_bytes32(&input[32..64]);
    let y = as_bytes32(&input[64..96]);
    let proof = as_bytes48(&input[144..192]);
    if !verify_kzg_proof(commitment, z, y, proof) {
        return Err(Error::KzgVerifyProofFailed);
    }

    Ok((GAS_COST, RETURN_VALUE.to_vec()))
}

/// `VERSIONED_HASH_VERSION_KZG ++ sha256(commitment)[1..]`
fn kzg_to_versioned_hash(commitment: &[u8]) -> [u8; 32] {
    let mut hash: [u8; 32] = Sha256::digest(commitment).into();
    hash[0] = VERSIONED_HASH_VERSION_KZG;
    hash
}

fn verify_kzg_proof(commitment: &Bytes48, z: &Bytes32, y: &Bytes32, proof: &Bytes48) -> bool {
    let mut ok = false;
    let ret =
        unsafe { bindings::verify_kzg_proof(&mut ok, commitment, z, y, proof, get_kzg_settings()) };
    debug_assert!(
        ret == CkzgError::C_KZG_OK,
        "verify_kzg_proof returned an error: {ret:?}"
    );
    ok
}

fn get_kzg_settings() -> &'static KzgSettings {
    static SETTINGS: OnceCell<KzgSettings> = OnceCell::new();
    SETTINGS.get_or_init(|| {
        c_kzg::KzgSettings::load_trusted_setup(
            generated_settings::G1_POINTS,
            generated_settings::G2_POINTS,
        )
        .expect("failed to load trusted setup")
    })
}

#[inline(always)]
#[cfg_attr(debug_assertions, track_caller)]
fn as_array<const N: usize>(bytes: &[u8]) -> &[u8; N] {
    debug_assert_eq!(bytes.len(), N);
    // SAFETY: Length is checked above
    unsafe { &*bytes.as_ptr().cast() }
}

#[inline(always)]
#[cfg_attr(debug_assertions, track_caller)]
fn as_bytes32(bytes: &[u8]) -> &Bytes32 {
    // SAFETY: `#[repr(C)] Bytes32([u8; 32])`
    unsafe { *as_array::<32>(bytes).as_ptr().cast() }
}

#[inline(always)]
#[cfg_attr(debug_assertions, track_caller)]
fn as_bytes48(bytes: &[u8]) -> &Bytes48 {
    // SAFETY: `#[repr(C)] Bytes48([u8; 48])`
    unsafe { *as_array::<48>(bytes).as_ptr().cast() }
}

#[cfg(test)]
mod tests {
    use super::*;
    use revm_primitives::U256;

    #[test]
    fn bls_modulus() {
        let modulus =
            "52435875175126190479447740508185965837690552500527637822603658699938581184513";
        let modulus = modulus.parse::<U256>().unwrap();
        assert_eq!(modulus.to_be_bytes(), *BLS_MODULUS);
    }

    #[test]
    fn return_value() {
        let elements = U256::from(FIELD_ELEMENTS_PER_BLOB);
        let result = [elements.to_be_bytes(), *BLS_MODULUS].concat();
        assert_eq!(RETURN_VALUE[..], result);
    }
}
