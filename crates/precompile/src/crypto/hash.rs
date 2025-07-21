//! Hash function implementations

pub mod constants {
    //! Constants for hash functions

    /// SHA-256 output length in bytes
    pub const SHA256_LENGTH: usize = 32;

    /// RIPEMD-160 output length in bytes  
    pub const RIPEMD160_LENGTH: usize = 20;
}

use sha2::Digest;

/// Compute SHA-256 hash
#[inline(always)]
pub fn sha256(input: &[u8]) -> [u8; constants::SHA256_LENGTH] {
    let output = sha2::Sha256::digest(input);
    output.into()
}

/// Compute RIPEMD-160 hash (padded to 32 bytes)
#[inline(always)]
pub fn ripemd160(input: &[u8]) -> [u8; 32] {
    let mut hasher = ripemd::Ripemd160::new();
    hasher.update(input);

    let mut output = [0u8; 32];
    hasher.finalize_into((&mut output[12..]).into());
    output
}
