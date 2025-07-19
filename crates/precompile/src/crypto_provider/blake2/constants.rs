//! Blake2 cryptographic constants
//!
//! These constants define the sizes of various Blake2 primitives.

/// Blake2 compression function state length (8 u64 values).
pub const STATE_LENGTH: usize = 8;

/// Blake2 compression function message block length in bytes.
pub const MESSAGE_LENGTH: usize = 128;
