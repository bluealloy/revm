//! secp256r1 (P-256) cryptographic implementations for the crypto provider

/// secp256r1 cryptographic constants
pub mod constants;
pub use constants::*;

/// secp256r1 (P-256) signature verification.
///
/// Verifies a secp256r1 signature.
///
/// # Arguments
/// * `msg` - The message hash (MESSAGE_HASH_LENGTH bytes)
/// * `sig` - The signature (SIGNATURE_LENGTH bytes: r || s)
/// * `pk` - The uncompressed public key (PUBKEY_LENGTH bytes: 0x04 || x || y)
///
/// # Returns
/// `true` if the signature is valid, `false` otherwise.
pub fn verify(
    msg: &[u8; MESSAGE_HASH_LENGTH],
    sig: &[u8; SIGNATURE_LENGTH],
    pk: &[u8; PUBKEY_LENGTH],
) -> bool {
    use p256::ecdsa::{signature::hazmat::PrehashVerifier, Signature, VerifyingKey};

    // Can fail only if the input is not exact length.
    let signature = match Signature::from_slice(sig) {
        Ok(sig) => sig,
        Err(_) => return false,
    };

    // Can fail if the input is not valid, so we have to propagate the error.
    let public_key = match VerifyingKey::from_sec1_bytes(pk) {
        Ok(pk) => pk,
        Err(_) => return false,
    };

    public_key.verify_prehash(msg, &signature).is_ok()
}
