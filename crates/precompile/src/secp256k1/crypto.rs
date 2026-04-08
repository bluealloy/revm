//! Pure cryptographic implementation for secp256k1 ecrecover.
//!
//! This module isolates the cryptographic logic from the precompile runner,
//! containing only the signature recovery implementation without EVM-specific concerns.

// Select the correct implementation based on the enabled features.
cfg_if::cfg_if! {
    if #[cfg(feature = "secp256k1")] {
        pub use super::bitcoin_secp256k1::ecrecover;
    } else {
        pub use super::k256::ecrecover;
    }
}

/// Recover an Ethereum address from a secp256k1 ECDSA signature.
///
/// Returns `Some(address)` as a 32-byte array (12 zero bytes + 20 address bytes)
/// if recovery succeeds, or `None` on failure.
pub(crate) fn ecrecover_bytes(sig: &[u8; 64], recid: u8, msg: &[u8; 32]) -> Option<[u8; 32]> {
    match ecrecover(sig.into(), recid, msg.into()) {
        Ok(address) => Some(address.0),
        Err(_) => None,
    }
}
