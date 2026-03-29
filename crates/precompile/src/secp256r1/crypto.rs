//! Pure cryptographic implementation for secp256r1 (P-256) signature verification.
//!
//! This module isolates the cryptographic logic from the precompile runner,
//! containing only the signature verification implementation without EVM-specific concerns.

/// Verify a secp256r1 (P-256) ECDSA signature over a prehashed message.
///
/// Returns `Some(())` if the signature is valid, `None` otherwise.
pub(crate) fn verify_signature(msg: &[u8; 32], sig: &[u8; 64], pk: &[u8; 64]) -> Option<()> {
    cfg_if::cfg_if! {
        if #[cfg(feature = "p256-aws-lc-rs")] {
            use aws_lc_rs::{digest, signature::{self, UnparsedPublicKey}};

            // Construct a Digest from the raw prehashed message bytes.
            let digest = digest::Digest::import_less_safe(msg, &digest::SHA256).ok()?;

            // Build uncompressed public key: 0x04 || x || y
            let mut pubkey_bytes = [0u8; 65];
            pubkey_bytes[0] = 0x04;
            pubkey_bytes[1..].copy_from_slice(pk);

            let public_key = UnparsedPublicKey::new(&signature::ECDSA_P256_SHA256_FIXED, &pubkey_bytes);

            public_key.verify_digest(&digest, sig).ok()
        } else {
            use p256::{
                ecdsa::{signature::hazmat::PrehashVerifier, Signature, VerifyingKey},
                EncodedPoint,
            };

            // Can fail only if the input is not exact length.
            let signature = Signature::from_slice(sig).ok()?;
            // Decode the public key bytes (x,y coordinates) using EncodedPoint
            let encoded_point = EncodedPoint::from_untagged_bytes(&(*pk).into());
            // Create VerifyingKey from the encoded point
            let public_key = VerifyingKey::from_encoded_point(&encoded_point).ok()?;

            public_key.verify_prehash(msg, &signature).ok()
        }
    }
}
