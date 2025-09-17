//! Blake2 cryptographic operations
//!
//! This module contains the pure cryptographic implementations for blake2 precompiles.
//! These functions are called by the Crypto trait.

/// Blake2 compression function
pub fn blake2_compress(rounds: u32, h: &mut [u64; 8], m: [u64; 16], t: [u64; 2], f: bool) {
    crate::blake2::algo::compress(rounds as usize, h, m, t, f);
}
