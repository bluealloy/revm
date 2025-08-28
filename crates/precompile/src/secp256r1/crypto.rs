//! secp256r1 cryptographic operations
//! 
//! This module contains the pure cryptographic implementations for secp256r1 precompiles.
//! These functions are called by the Crypto trait.

/// secp256r1 (P-256) signature verification
pub fn verify_signature(msg: &[u8; 32], sig: &[u8; 64], pk: &[u8; 64]) -> Option<()> {
    crate::secp256r1::verify_signature(*msg, *sig, *pk)
}
