//! BLS12-381 cryptographic operations
//! 
//! This module contains the pure cryptographic implementations for BLS12-381 precompiles.
//! These functions are called by the Crypto trait and delegate to the appropriate backend.

use super::{G1Point, G1PointScalar, G2Point, G2PointScalar};
use crate::PrecompileError;

/// BLS12-381 G1 point addition
pub fn g1_add(a: G1Point, b: G1Point) -> Result<[u8; 96], PrecompileError> {
    super::crypto_backend::p1_add_affine_bytes(a, b)
}

/// BLS12-381 G1 multi-scalar multiplication
pub fn g1_msm(
    pairs: &mut dyn Iterator<Item = Result<G1PointScalar, PrecompileError>>,
) -> Result<[u8; 96], PrecompileError> {
    // Convert the iterator to the expected format
    let converted_pairs = pairs.map(|result| {
        result.map(|(point, scalar)| (point, scalar))
    });
    
    super::crypto_backend::p1_msm_bytes(converted_pairs)
}

/// BLS12-381 G2 point addition
pub fn g2_add(a: G2Point, b: G2Point) -> Result<[u8; 192], PrecompileError> {
    super::crypto_backend::p2_add_affine_bytes(a, b)
}

/// BLS12-381 G2 multi-scalar multiplication
pub fn g2_msm(
    pairs: &mut dyn Iterator<Item = Result<G2PointScalar, PrecompileError>>,
) -> Result<[u8; 192], PrecompileError> {
    // Convert the iterator to the expected format
    let converted_pairs = pairs.map(|result| {
        result.map(|(point, scalar)| (point, scalar))
    });
    
    super::crypto_backend::p2_msm_bytes(converted_pairs)
}

/// BLS12-381 pairing check
pub fn pairing_check(pairs: &[(G1Point, G2Point)]) -> Result<bool, PrecompileError> {
    Ok(super::crypto_backend::pairing_check_bytes(pairs)?)
}

/// BLS12-381 map field element to G1
pub fn fp_to_g1(fp: &[u8; 48]) -> Result<[u8; 96], PrecompileError> {
    super::crypto_backend::map_fp_to_g1_bytes(fp)
}

/// BLS12-381 map field element to G2
pub fn fp2_to_g2(fp2: ([u8; 48], [u8; 48])) -> Result<[u8; 192], PrecompileError> {
    super::crypto_backend::map_fp2_to_g2_bytes(&fp2.0, &fp2.1)
}
