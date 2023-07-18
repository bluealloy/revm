use core::ops::Mul;

use alloc::vec::Vec;
use revm_primitives::{PrecompileResult, StandardPrecompileFn};
use sha2::{Digest, Sha256};
use ark_ec::{Group, pairing::Pairing};
use ark_bls12_381::{G1Affine, G2Affine, Bls12_381, Fr, G1Projective, G2Projective};
use ark_serialize::CanonicalDeserialize;

// Maybe use blst crate instead of ark_bls12_381 after we get it working
// use blst::*;

// use crate::{Precompile, PrecompileAddress};

pub const POINT_EVALUATION_PRECOMPILE: PrecompileAddress = PrecompileAddress(
    crate::u64_to_b160(12),
    Precompile::Standard(point_evaluation_run as StandardPrecompileFn),
);

const G1_POINT_AT_INFINITY: [u8; 48] = [
    0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

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
type BLSFieldElement = [u8; 48];

// Custom type for G1Point
// Validation: Perform BLS standard's "KeyValidate" check but do allow the identity point
type G1Point = G1Affine;
// Custom type for G2Point
type G2Point = G2Affine;
// Custom type for KZGCommitment
// Validation: Perform BLS standard's "KeyValidate" check but do allow the identity point
type KZGCommitment = [u8; 48];
// Custom type for KZGProof
type KZGProof = [u8; 48];
// Custom type for Polynomial
// A polynomial in evaluation form
pub struct Polynomial {
    coefficients: [BLSFieldElement; FIELD_ELEMENTS_PER_BLOB], 
}
// Custom type for Blob
// A basic blob data
pub struct Blob {
    data: [u8; BYTES_PER_FIELD_ELEMENT * FIELD_ELEMENTS_PER_BLOB],
}
// Custom type for Versioned Hash
type VersionedHash = [u8; 32];


pub fn point_evaluation_run(input: &[u8], gas_limit: u64) -> PrecompileResult {
    // The data is encoded as follows: versioned_hash | z | y | commitment | proof | with z and y being padded 32 byte big endian values
    assert!(input.len() == 192);
    let versioned_hash = &input[0..32];
    let z = &input[32..64];
    let y = &input[64..96];
    let commitment =  &input[96..144];
    let proof = &input[144..192];

    // Verify commitment matches versioned_hash
    assert!(kzg_to_versioned_hash(commitment) == versioned_hash);

    // Verify KZG proof with z and y in big endian format
    unsafe {
        assert!(verify_kzg_proof(commitment, z, y, proof));
    }

    let bytes: [u8; core::mem::size_of::<usize>()] = FIELD_ELEMENTS_PER_BLOB.to_ne_bytes();
    let mut result = Vec::from(bytes); // The first bytes of the result are the FIELD_ELEMENTS_PER_BLOB
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

pub unsafe fn verify_kzg_proof(_commitment: &[u8], _z: &[u8], _y: &[u8], _proof: &[u8]) -> bool {
    // Step 1)
    // convert byte slices into BLS types 
    // maybe have to deseFrs()
    let commitment = G1Projective::deserialize_compressed_unchecked(&_commitment[..]).unwrap();
    let proof = G1Projective::deserialize_compressed_unchecked(&_proof[..]).unwrap();
    
    let z: Fr = CanonicalDeserialize::deserialize_compressed_unchecked(_z).unwrap();
    let y: Fr = CanonicalDeserialize::deserialize_compressed_unchecked(_y).unwrap();
    
    let gen1 = G1Projective::generator();
    let gen2 = G2Projective::generator();

    // sG2 - zG2
    // TODO: This should be KZG_SETUP_G2[1] + gen2.mul(-z)
    let s_minus_z = gen2.mul(-z);
    // C - yG1
    let commitment_minus_y = commitment - gen1.mul(y);
    
    let lhs = Bls12_381::pairing(commitment_minus_y, gen2);
    let rhs = Bls12_381::pairing(proof, s_minus_z);

    lhs == rhs
}


// Bit reversal permutation
pub fn bit_reversal_permutation<T: Clone + Default >(sequence: &Vec<T>) -> Vec<T> {
    let n = sequence.len();
    if n <= 1 {
        return sequence.clone();
    } 
    let bit_length = n.next_power_of_two().trailing_zeros();
    (0..n).map(|i| sequence[reverse_bits(i as u32, bit_length) as usize].clone()).collect()
}

pub fn is_power_of_two(value: u32) -> bool {
    value > 0 && (value & (value - 1)) == 0
}

pub fn reverse_bits(n: u32, order: u32) -> u32 {
    let mut n = n;
    let mut result = 0;
    let mut count = order;

    while count > 0 {
        result <<= 1;
        result |= n & 1;
        n >>= 1;
        count -= 1;
    }
    result
}

pub fn bytes_to_bls_field_element(bytes: &[u8]) -> BLSFieldElement {
    assert!(bytes.len() == 48);
    let mut result: BLSFieldElement = [0u8; 48];
    result.copy_from_slice(bytes);
    result
}

fn bytes_to_kzg_proof(b: &[u8]) -> KZGProof {
    // """
    // Convert untrusted bytes into a trusted and validated KZGProof.
    // """
    

    validate_kzg_g1(b);
    return kzgproof(b)
}
fn validate_kzg_g1(bytes: &[u8]) {
    if bytes == G1_POINT_AT_INFINITY {
        return
    }
    assert!(validate_kzg_point(bytes));
}

fn validate_kzg_point(bytes: &[u8]) -> bool {
    if bytes == G1_POINT_AT_INFINITY {
        true;
    }
    G1Projective::deserialize_compressed(bytes).is_ok()
}

fn kzgproof(bytes: &[u8]) -> KZGProof {
    let array: [u8; 48] = bytes.try_into().expect("Slice with incorrect length");
    array
}

fn bytes_to_bls_field(b: &[u8]) -> BLSFieldElement {
    
    // Convert untrusted bytes to a trusted and validated BLS scalar field element.
    // This function does not accept inputs greater than the BLS modulus.
    let field_element = bytes_to_bls_field_element(b);
    let modulus_field_element = bytes_to_bls_field_element(BLS_MODULUS.as_slice());
    assert!(field_element < modulus_field_element);
    field_element
    
} 

mod tests {
    use super::*;

    #[test]
    fn bit_reversal_permutation_empty() {
        let v: Vec<i32> = Vec::new();
        let result = bit_reversal_permutation(&v);
        assert_eq!(result, Vec::new());
    }
    #[test]
    fn bit_reversal_permutation_single_element() {
        let v = vec![1];
        let result = bit_reversal_permutation(&v);
        assert_eq!(result, vec![1]);
    }

    #[test]
    fn test_bit_reversal_permutation_multiple_elements() {
        let v = vec![1, 2, 3, 4];
        let result = bit_reversal_permutation(&v);
        assert_eq!(result, vec![1, 3, 2, 4]);
    }

    #[test]
    fn test_bit_reversal_permutation_even_length() {
        let v = vec![1, 2, 3, 4, 5, 6];
        let result = bit_reversal_permutation(&v);
        assert_eq!(result, vec![1, 5, 3, 0, 2, 6, 4, 0]);
    }

    #[test]
    fn test_bit_reversal_permutation_odd_length() {
        let v = vec![1, 2, 3, 4, 5];
        let result = bit_reversal_permutation(&v);
        assert_eq!(result, vec![1, 3, 2, 5, 4]);
    }

    #[test]
    fn test_bit_reversal_permutation_large_input() {
        let v: Vec<u32> = (1..=10000).collect();
        let result = bit_reversal_permutation(&v);
        // Assert with a known value or pattern.
    }

    #[test]
    fn test_bit_reversal_permutation_different_types() {
        let v = vec!["a", "b", "c", "d"];
        let result = bit_reversal_permutation(&v);
        assert_eq!(result, vec!["a", "c", "b", "d"]);
    }

    #[test]
    fn kzg_to_versioned_hash() {
        // let commitment = [0x01, 0x02];
        // let hashed_commitment = super::kzg_to_versioned_hash(&commitment);
        // assert_eq!(
        //     hashed_commitment,
        //     [
        //         1, 40, 113, 254, 226, 16, 251, 134, 25, 41, 30, 174, 161, 148, 88, 28, 189, 37, 49,
        //         228, 178, 55, 89, 210, 37, 246, 128, 105, 35, 246, 50, 34
        //     ]
        // );
        todo!();
    }

    #[test]
    fn verify_kzg_proof() {
        todo!();
    }

    #[test]
    fn point_evaluation_run() {
        // Test the assertion in the run too
        todo!();
    }
}
