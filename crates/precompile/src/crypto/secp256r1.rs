//! secp256r1 (P-256) signature verification

pub mod constants {
    //! Constants for secp256r1 (P-256) operations

    /// Length of the message hash (32 bytes)
    pub const MESSAGE_HASH_LENGTH: usize = 32;

    /// Length of the signature (64 bytes: r || s)
    pub const SIGNATURE_LENGTH: usize = 64;

    /// Length of the uncompressed public key (64 bytes: x || y)
    pub const PUBKEY_LENGTH: usize = 64;
}

use p256::{
    ecdsa::{signature::hazmat::PrehashVerifier, Signature, VerifyingKey},
    EncodedPoint,
};

/// Verify a secp256r1 signature
///
/// # Arguments
/// * `msg` - The message hash (32 bytes)
/// * `sig` - The signature (64 bytes: r || s)  
/// * `pk` - The uncompressed public key (64 bytes: x || y)
///
/// # Returns
/// `Some(())` if the signature is valid, `None` otherwise
pub fn verify_signature(
    msg: &[u8; constants::MESSAGE_HASH_LENGTH],
    sig: &[u8; constants::SIGNATURE_LENGTH],
    pk: &[u8; constants::PUBKEY_LENGTH],
) -> Option<()> {
    // Can fail only if the input is not exact length.
    let signature = Signature::from_slice(sig).ok()?;
    // Decode the public key bytes (x,y coordinates) using EncodedPoint
    let encoded_point = EncodedPoint::from_untagged_bytes(pk.into());
    // Create VerifyingKey from the encoded point
    let public_key = VerifyingKey::from_encoded_point(&encoded_point).ok()?;

    public_key.verify_prehash(msg, &signature).ok()
}
