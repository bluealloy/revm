use crate::{Precompile, PrecompileAddress};
use c_kzg::*;
use revm_primitives::{PrecompileResult, StandardPrecompileFn};

pub const POINT_EVALUATION_PRECOMPILE: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b160(12),
    Precompile::Standard(point_evaluation_run as StandardPrecompileFn),
);

/// `BLS_MODULUS: = 52435875175126190479447740508185965837690552500527637822603658699938581184513`
/// in big endian format
const BLS_MODULUS: [u8; 32] = [
    115, 237, 167, 83, 41, 157, 125, 72, 51, 57, 216, 8, 9, 161, 216, 5, 83, 189, 164, 2, 255, 254,
    91, 254, 255, 255, 255, 255, 0, 0, 0, 1,
];

pub fn point_evaluation_run(input: &[u8], gas_limit: u64) -> PrecompileResult {
    // The data is encoded as follows: versioned_hash | z | y | commitment | proof | with z and y being padded 32 byte big endian values
    assert!(input.len() == 192);

    // We can always be sure that these will be 48 bytes so this unwrap should be okay
    let z = Bytes32::from_bytes(&input[32..64]).unwrap();
    let y = Bytes32::from_bytes(&input[64..96]).unwrap();
    let commitment = Bytes48::from_bytes(&input[96..144]).unwrap();
    let versioned_hash = Bytes48::from_bytes(&input[0..32]).unwrap();
    let proof = Bytes48::from_bytes(&input[144..192]).unwrap();
    let kzg_settings = c_kzg::KzgSettings::load_trusted_setup_file(
        "crates/precompile/src/trusted_setup4.txt".into(),
    )
    .unwrap();

    // Verify commitment matches versioned_hash
    assert!(commitment == versioned_hash);
    // Verify KZG proof with z and y in big endian format
    assert!(c_kzg::KzgProof::verify_kzg_proof(commitment, z, y, proof, &kzg_settings).unwrap());


    // # Return FIELD_ELEMENTS_PER_BLOB and BLS_MODULUS as padded 32 byte big endian values

    // Convert FIELD_ELEMENTS_PER_BLOB to big-endian bytes and pad to 32 bytes
    let mut field_elements_bytes = [0u8; 32];
    let field_elements_bytes_small = c_kzg::FIELD_ELEMENTS_PER_BLOB.to_be_bytes();
    field_elements_bytes[(32 - field_elements_bytes_small.len())..].copy_from_slice(&field_elements_bytes_small);

    // Concatenate the byte arrays
    let mut result = [0u8; 64];
    result[0..32].copy_from_slice(&field_elements_bytes);
    result[32..64].copy_from_slice(&BLS_MODULUS);

    Ok((gas_limit, result.to_vec()))
}
