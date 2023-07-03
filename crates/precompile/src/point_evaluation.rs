use core::str::FromStr;

use alloc::vec::Vec;
use num::{BigUint, FromPrimitive};
use revm_primitives::{PrecompileResult, StandardPrecompileFn};
use sha2::{Digest, Sha256};

use crate::{Precompile, PrecompileAddress};

pub const POINT_EVALUATION_PRECOMPILE: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b160(12),
    Precompile::Standard(point_evaluation_run as StandardPrecompileFn),
);

const FIELD_ELEMENTS_PER_BLOB: u32 = 4096;
// Modulus is 381 bits which is greater than 256 bits so need to find better type
const BLS_MODULUS: &str =
    "52435875175126190479447740508185965837690552500527637822603658699938581184513";
const BLOB_COMMITMENT_VERSION_KZG: u8 = 0x01;

pub fn point_evaluation_run(input: &[u8], gas_limit: u64) -> PrecompileResult {
    // The data is encoded as follows: versioned_hash | z | y | commitment | proof | with z and y being padded 32 byte big endian values
    assert!(input.len() == 192);
    let versioned_hash = &input[0..32];
    let z = &input[32..64];
    let y = &input[64..96];
    let commitment = &input[96..144];
    let proof = &input[144..192];

    // Verify commitment matches versioned_hash
    assert!(_kzg_to_versioned_hash(commitment) == versioned_hash);

    // Verify KZG proof with z and y in big endian format
    assert!(_verify_kzg_proof(commitment, z, y, proof));

    // Convert FIELD_ELEMENTS_PER_BLOB to BigUint
    let field_elements_big_uint = BigUint::from_u32(FIELD_ELEMENTS_PER_BLOB).unwrap();
    let mut field_elements_bytes = vec![0; 32];
    let bytes_be = field_elements_big_uint.to_bytes_be();
    field_elements_bytes[32 - bytes_be.len()..].copy_from_slice(&bytes_be);

    // Convert BLS_MODULUS to BigUint
    let bls_modulus_big_uint = BigUint::from_str(BLS_MODULUS).expect("Failed to parse BLS_MODULUS");
    let mut bls_modulus_bytes = vec![0; 32];
    let bytes_be = bls_modulus_big_uint.to_bytes_be();
    bls_modulus_bytes[32 - bytes_be.len()..].copy_from_slice(&bytes_be);

    // Concatenate both byte arrays
    let mut result = field_elements_bytes;
    result.extend(bls_modulus_bytes);
    Ok((gas_limit, result))
}

pub fn _kzg_to_versioned_hash(commitment: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(commitment);
    let mut hash = hasher.finalize().to_vec();

    // Skip the first byte
    hash.drain(0..1);

    // Prepend the version marker
    let mut result = vec![BLOB_COMMITMENT_VERSION_KZG];
    result.append(&mut hash);

    result
}
pub fn _verify_kzg_proof(_commitment: &[u8], _z: &[u8], _y: &[u8], _proof: &[u8]) -> bool {
    todo!();
}
