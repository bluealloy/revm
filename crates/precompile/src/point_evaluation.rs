use num::BigUint;
use revm_primitives::U256;

const FIELD_ELEMENTS_PER_BLOB: u32 = 4096;
const BLS_MODULUS: U256 = 52435875175126190479447740508185965837690552500527637822603658699938581184513;
const BLOB_COMMITMENT_VERSION_KZG: u8 = 0x01;
/// Verify p(z) = y given commitment that corresponds to the polynomial p(x) and a KZG proof.
/// Also verify that the provided commitment matches the provided versioned_hash.
pub fn point_evaluation_precompile(input: &[u8]) -> &[u8]{

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

    // Return FIELD_ELEMENTS_PER_BLOB and BLS_MODULUS as padded 32 byte big endian values
    let field_elements_per_blob = BigUint::from_bytes_be(&[FIELD_ELEMENTS_PER_BLOB.try_into().unwrap()]);
    let bls_modulus = BigUint::from_bytes_be(&[BLS_MODULUS]);
    
    let mut bytes = field_elements_per_blob.to_bytes_be();
    bytes.extend(bls_modulus.to_bytes_be());
    
    return bytes;}
pub fn kzg_to_versioned_hash(commitment: KZGCommitment) -> VersionedHash {
    return BLOB_COMMITMENT_VERSION_KZG + sha256(commitment)[1:]
}
