//! Crypto provider interface for precompiles
//!
//! This module defines a trait that abstracts cryptographic operations
//! for various precompiles, allowing different implementations to be used.

use crate::PrecompileError;

/// Trait for cryptographic operations used by precompiles.
pub trait CryptoProvider {
    /// BN128 elliptic curve addition.
    /// 
    /// Takes two points on the BN128 G1 curve and returns their sum.
    /// 
    /// # Arguments
    /// * `p1` - First point as 64 bytes (x, y coordinates, 32 bytes each)
    /// * `p2` - Second point as 64 bytes (x, y coordinates, 32 bytes each)
    /// 
    /// # Returns
    /// The sum of the two points as 64 bytes, or an error if the points are invalid.
    fn bn128_add(&self, p1: &[u8; 64], p2: &[u8; 64]) -> Result<[u8; 64], PrecompileError>;
}

/// Default crypto provider using the existing implementations
#[derive(Debug)]
pub struct DefaultCryptoProvider;

impl CryptoProvider for DefaultCryptoProvider {
    fn bn128_add(&self, p1: &[u8; 64], p2: &[u8; 64]) -> Result<[u8; 64], PrecompileError> {
        // Use the existing backend implementation
        cfg_if::cfg_if! {
            if #[cfg(feature = "bn")]{
                crate::bn128::substrate::g1_point_add(p1, p2)
            } else {
                crate::bn128::arkworks::g1_point_add(p1, p2)
            }
        }
    }
}

// For now, we use a static default provider
// In a more complete implementation, this could be made configurable
static DEFAULT_PROVIDER: DefaultCryptoProvider = DefaultCryptoProvider;

/// Get the crypto provider instance.
/// 
/// Currently returns the default provider. In future, this could be made configurable.
pub fn get_crypto_provider() -> &'static dyn CryptoProvider {
    &DEFAULT_PROVIDER
}