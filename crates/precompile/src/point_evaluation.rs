use alloc::vec::Vec;
use revm_primitives::{PrecompileResult, StandardPrecompileFn};
use sha2::{Digest, Sha256};
use ark_bls12_381::{G1Affine, G2Affine, Bls12_381, Fr, G1Projective as G1, G2Projective as G2};
use ark_ff::BigInteger320;

use crate::{Precompile, PrecompileAddress};

pub const POINT_EVALUATION_PRECOMPILE: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b160(12),
    Precompile::Standard(point_evaluation_run as StandardPrecompileFn),
);

const FIELD_ELEMENTS_PER_BLOB: usize = 4096;
/// in big endian format
// const FIELD_ELEMENTS_PER_BLOB: [u8; 4] = [0, 0, 16, 0];
/// `BLS_MODULUS: = 52435875175126190479447740508185965837690552500527637822603658699938581184513`
/// in big endian format
const BLS_MODULUS: [u8; 32] = [
    115, 237, 167, 83, 41, 157, 125, 72, 51, 57, 216, 8, 9, 161, 216, 5, 83, 189, 164, 2, 255, 254,
    91, 254, 255, 255, 255, 255, 0, 0, 0, 1,
];
const BLOB_COMMITMENT_VERSION_KZG: u8 = 1;
// maybe needs to be u64
const BYTES_PER_FIELD_ELEMENT: usize = 32;

// Alias for BLSFieldElement
// Validation: x < BLS_MODULUS
pub struct BLSFieldElement {
    bytes: [u8; 48],
}

// Custom type for G1Point
// Validation: Perform BLS standard's "KeyValidate" check but do allow the identity point
pub struct G1Point {
    point: G1Affine, // or another appropriate type depending on your exact requirements
}

// Custom type for G2Point
pub struct G2Point {
    point: G2Affine, // or another appropriate type depending on your exact requirements
}

// Custom type for KZGCommitment
// Validation: Perform BLS standard's "KeyValidate" check but do allow the identity point
pub struct KZGCommitment {
    commitment: [u8; 48], // or another appropriate type depending on your exact requirements
}

// Custom type for KZGProof
pub struct KZGProof {
    proof: [u8; 48], // or another appropriate type depending on your exact requirements
}

// Custom type for Polynomial
// A polynomial in evaluation form
pub struct Polynomial {
    coefficients: [BLSFieldElement; FIELD_ELEMENTS_PER_BLOB], 
}

// Custom type for Blob
// A basic blob data
pub struct Blob {
    data: [u8; BYTES_PER_FIELD_ELEMENT * FIELD_ELEMENTS_PER_BLOB], // Replace BYTES_PER_FIELD_ELEMENT and FIELD_ELEMENTS_PER_BLOB with the appropriate size
}


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
    result.extend(Vec::from(BLS_MODULUS)); // Concatenate the BLS_MODULUS to the result

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

// impl BlsFieldElement {
//     pub fn from_bytes(bytes: &[u8]) -> Self {
//         assert!(bytes.len() == 32);
//         let mut bytes = bytes.to_vec();
//         bytes.reverse();
//         BlsFieldElement(U256::from_little_endian(&bytes))
//     }
// }
pub fn verify_kzg_proof(_commitment: &[u8], _z: &[u8], _y: &[u8], _proof: &[u8]) -> bool {

    // let bls_modulus_minus_z = Field::zero() - &z;
    // let bls_modulus_minus_y = Fr::zero() - &y;

    // let x_minus_z = G2::one().add(&kzg_setup_g2.mul(&bls_modulus_minus_z.into_repr()));
    // let p_minus_y = commitment.add(&G1::one().mul(&bls_modulus_minus_y.into_repr()));

    // let neg_g2 = G2::one().neg();

    // let pairing_check_1 = Bls12_381::pairing(p_minus_y.into_affine(), neg_g2.into_affine());
    // let pairing_check_2 = Bls12_381::pairing(proof.com.into_affine(), x_minus_z.into_affine());

    // pairing_check_1 == pairing_check_2
    todo!()
}


// Bit reversal permutation
pub fn bit_reversal_permutation<T: Clone>(sequence: &Vec<T>) -> Vec<T> {
    let n = sequence.len();
    let bit_length = n.next_power_of_two().trailing_zeros();
    (0..n).map(|i| sequence[reverse_bits(i as u32, bit_length) as usize].clone()).collect()
}

pub fn is_power_of_two(value: u32) -> bool {
    value > 0 && (value & (value - 1)) == 0
}

pub fn reverse_bits(n: u32, order: u32) -> u32 {
    let mut result = n;
    let mut n = n;
    for _ in 0..order {
        result <<= 1;
        result |= n & 1;
        n >>= 1;
    }
    result >> (32 - order)
}




