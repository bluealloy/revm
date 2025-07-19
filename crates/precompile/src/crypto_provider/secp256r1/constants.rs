//! secp256r1 (P-256) cryptographic constants
//!
//! These constants define the sizes of various secp256r1 cryptographic primitives.

/// secp256r1 signature length in bytes (r || s).
pub const SIGNATURE_LENGTH: usize = 64;

/// secp256r1 public key length in bytes (uncompressed: 0x04 || x || y).
pub const PUBKEY_LENGTH: usize = 65;

/// secp256r1 private key length in bytes.
pub const PRIVATE_KEY_LENGTH: usize = 32;

/// Message hash length in bytes.
pub const MESSAGE_HASH_LENGTH: usize = 32;
