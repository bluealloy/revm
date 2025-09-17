//! Hash cryptographic operations
//!
//! This module contains the pure cryptographic implementations for hash precompiles.
//! These functions are called by the Crypto trait.

/// Compute SHA-256 hash
pub fn sha256(input: &[u8]) -> [u8; 32] {
    use sha2::Digest;
    let output = sha2::Sha256::digest(input);
    output.into()
}

/// Compute RIPEMD-160 hash
pub fn ripemd160(input: &[u8]) -> [u8; 32] {
    use ripemd::Digest;
    let mut hasher = ripemd::Ripemd160::new();
    hasher.update(input);

    let mut output = [0u8; 32];
    hasher.finalize_into((&mut output[12..]).into());
    output
}
