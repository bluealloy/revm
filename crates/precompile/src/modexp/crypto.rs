//! Modular exponentiation cryptographic operations
//!
//! This module contains the pure cryptographic implementations for modexp precompiles.
//! These functions are called by the Crypto trait.

use crate::PrecompileError;
use std::vec::Vec;

/// Modular exponentiation
pub fn modexp(base: &[u8], exp: &[u8], modulus: &[u8]) -> Result<Vec<u8>, PrecompileError> {
    Ok(crate::modexp::modexp(base, exp, modulus))
}
