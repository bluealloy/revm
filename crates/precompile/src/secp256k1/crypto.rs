//! secp256k1 cryptographic operations
//!
//! This module contains the pure cryptographic implementations for secp256k1 precompiles.
//! These functions are called by the Crypto trait.

use crate::PrecompileError;

/// secp256k1 ECDSA signature recovery
pub fn ecrecover(sig: &[u8; 64], recid: u8, msg: &[u8; 32]) -> Result<[u8; 32], PrecompileError> {
    crate::secp256k1::ecrecover_bytes(*sig, recid, *msg)
        .ok_or(PrecompileError::Secp256k1RecoverFailed)
}
