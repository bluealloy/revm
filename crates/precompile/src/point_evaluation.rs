use crate::{Precompile, PrecompileAddress};
use c_kzg::*;
use revm_primitives::{PrecompileResult, StandardPrecompileFn};

pub const POINT_EVALUATION_PRECOMPILE: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b160(12),
    Precompile::Standard(point_evaluation_run as StandardPrecompileFn),
);

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

    let result: [u8; core::mem::size_of::<usize>()] = FIELD_ELEMENTS_PER_BLOB.to_ne_bytes();
    // let mut result = Vec::from(bytes); // The first bytes of the result are the FIELD_ELEMENTS_PER_BLOB
    // result.extend(Vec::from(BLS_MODULUS)); // Concatenate the BLS_MODULUS to the result

    Ok((gas_limit, result.to_vec()))
}
