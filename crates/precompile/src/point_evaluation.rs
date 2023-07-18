use alloc::vec::Vec;
use revm_primitives::{PrecompileResult, StandardPrecompileFn};
use sha2::{Digest, Sha256};
use ark_bls12_381::{G1Affine, G2Affine, Bls12_381, Fr, G1Projective as G1, G2Projective as G2};
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize};

// Maybe use blst crate instead of ark_bls12_381 after we get it working
// use blst::*;

use crate::{Precompile, PrecompileAddress};

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
    // maybe have to deserializes
    let commitment: Fr = CanonicalDeserialize::deserialize_compressed_unchecked(_commitment).unwrap();
    // let z: 
    // P(z) = y
    // let thing = blst::blst_encode_to_g2(out, msg, msg_len, DST, DST_len, aug, aug_len)

    // let z_as_field_el = Fr::from(_z);
    // let y_as_field_el = Fr::from(_y);

    // let modulus_as_field_el = Fr::from(BLS_MODULUS);

    // let bls_modulus_minus_z = modulus_as_field_el - &z_as_field_el;
    // let bls_modulus_minus_y = modulus_as_field_el - &y_as_field_el;

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
    return KZGProof(b)
}
fn validate_kzg_g1(bytes: &[u8]) {
    if bytes == G1_POINT_AT_INFINITY {
        return
    }
    assert!(bls.KeyValidate(bytes))
}

fn KZGProof(bytes: &[u8]) -> KZGProof {
    KZGProof(bytes)
}

fn bytes_to_bls_field(b: &u8) -> Fr {
    /**
     * pyton impl
     *     """
    Convert untrusted bytes to a trusted and validated BLS scalar field element.
    This function does not accept inputs greater than the BLS modulus.
    """
    field_element = int.from_bytes(b, ENDIANNESS)
    assert field_element < BLS_MODULUS
    return BLSFieldElement(field_element)
     */
    
    // CanonicalDeserialize::deserialize_compressed_unchecked(b)
} 

mod tests {
    use super::*;

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
