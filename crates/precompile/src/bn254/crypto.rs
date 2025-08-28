//! BN254 cryptographic operations
//! 
//! This module contains the pure cryptographic implementations for BN254 precompiles.
//! These functions are called by the Crypto trait.

use crate::PrecompileError;

/// BN254 G1 point addition
pub fn g1_point_add(p1: &[u8], p2: &[u8]) -> Result<[u8; 64], PrecompileError> {
    crate::bn254::crypto_backend::g1_point_add(p1, p2)
}

/// BN254 G1 point scalar multiplication
pub fn g1_point_mul(point: &[u8], scalar: &[u8]) -> Result<[u8; 64], PrecompileError> {
    crate::bn254::crypto_backend::g1_point_mul(point, scalar)
}

/// BN254 pairing check
pub fn pairing_check(pairs: &[(&[u8], &[u8])]) -> Result<bool, PrecompileError> {
    crate::bn254::crypto_backend::pairing_check(pairs)
}
