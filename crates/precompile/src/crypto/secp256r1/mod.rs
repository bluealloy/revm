//! secp256r1 (P-256) signature verification

pub mod constants;

use p256::ecdsa::{signature::hazmat::PrehashVerifier, Signature, VerifyingKey};

/// Verify a secp256r1 signature
/// 
/// # Arguments
/// * `msg` - The message hash (32 bytes)
/// * `sig` - The signature (64 bytes: r || s)  
/// * `pk` - The uncompressed public key (65 bytes: 0x04 || x || y)
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
    // Can fail if the input is not valid, so we have to propagate the error.
    let public_key = VerifyingKey::from_sec1_bytes(pk).ok()?;

    public_key.verify_prehash(msg, &signature).ok()
}