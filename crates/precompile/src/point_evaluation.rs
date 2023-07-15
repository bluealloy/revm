use alloc::vec::Vec;
use revm_primitives::{PrecompileResult, StandardPrecompileFn};
use sha2::{Digest, Sha256};

use crate::{Precompile, PrecompileAddress};

pub const POINT_EVALUATION_PRECOMPILE: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b160(12),
    Precompile::Standard(point_evaluation_run as StandardPrecompileFn),
);

/// `FIELD_ELEMENTS_PER_BLOB = 4096`
/// in big endian format
const FIELD_ELEMENTS_PER_BLOB: [u8; 4] = [0, 0, 16, 0];
/// `BLS_MODULUS: = 52435875175126190479447740508185965837690552500527637822603658699938581184513`
/// in big endian format
const BLS_MODULUS: [u8; 32] = [
    115, 237, 167, 83, 41, 157, 125, 72, 51, 57, 216, 8, 9, 161, 216, 5, 83, 189, 164, 2, 255, 254,
    91, 254, 255, 255, 255, 255, 0, 0, 0, 1,
];
const BLOB_COMMITMENT_VERSION_KZG: u8 = 1;

pub fn point_evaluation_run(input: &[u8], gas_limit: u64) -> PrecompileResult {
    // The data is encoded as follows: versioned_hash | z | y | commitment | proof | with z and y being padded 32 byte big endian values
    assert!(input.len() == 192);
    let versioned_hash = &input[0..32];
    let z = &input[32..64];
    let y = &input[64..96];
    let commitment = &input[96..144];
    let proof = &input[144..192];

    // Verify commitment matches versioned_hash
    assert!(kzg_to_versioned_hash(commitment) == versioned_hash);

    // Verify KZG proof with z and y in big endian format
    assert!(verify_kzg_proof(commitment, z, y, proof));

    let mut result = Vec::from(FIELD_ELEMENTS_PER_BLOB); // The first bytes of the result are the FIELD_ELEMENTS_PER_BLOB
    let bls_modulus_bytes = Vec::from(BLS_MODULUS);
    result.extend(bls_modulus_bytes); // Concatenate the BLS_MODULUS to the result

    Ok((gas_limit, result))
}

pub fn kzg_to_versioned_hash(commitment: &[u8]) -> Vec<u8> {
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
pub fn verify_kzg_proof(_commitment: &[u8], _z: &[u8], _y: &[u8], _proof: &[u8]) -> bool {
    todo!();
}
