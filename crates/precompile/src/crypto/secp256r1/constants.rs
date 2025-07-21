//! Constants for secp256r1 (P-256) operations

/// Length of the message hash (32 bytes)
pub const MESSAGE_HASH_LENGTH: usize = 32;

/// Length of the signature (64 bytes: r || s)
pub const SIGNATURE_LENGTH: usize = 64;

/// Length of the uncompressed public key (65 bytes: 0x04 || x || y)
pub const PUBKEY_LENGTH: usize = 65;
