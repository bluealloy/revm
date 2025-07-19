//! Hash function constants
//!
//! These constants define the sizes of various hash function outputs.

/// SHA-256 hash output length in bytes.
pub const SHA256_LENGTH: usize = 32;

/// RIPEMD-160 hash output length in bytes.
pub const RIPEMD160_LENGTH: usize = 20;

/// Message hash length in bytes (commonly used across algorithms).
pub const MESSAGE_HASH_LENGTH: usize = 32;

/// Ethereum address length in bytes.
pub const ADDRESS_LENGTH: usize = 20;
