//! secp256k1 cryptographic constants
//!
//! These constants define the sizes of various secp256k1 cryptographic primitives.

/// secp256k1 signature length in bytes (r || s).
pub const SIGNATURE_LENGTH: usize = 64;

/// secp256k1 public key length in bytes (compressed).
pub const PUBKEY_COMPRESSED_LENGTH: usize = 33;

/// secp256k1 public key length in bytes (uncompressed).
pub const PUBKEY_UNCOMPRESSED_LENGTH: usize = 65;

/// secp256k1 private key length in bytes.
pub const PRIVATE_KEY_LENGTH: usize = 32;

/// Message hash length in bytes.
pub const MESSAGE_HASH_LENGTH: usize = 32;
