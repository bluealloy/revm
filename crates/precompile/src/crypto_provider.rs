//! Crypto provider interface for precompiles
//!
//! This module defines a trait that abstracts cryptographic operations
//! for various precompiles, allowing different implementations to be used.

use crate::PrecompileError;
use once_cell::race::OnceBox;
use std::boxed::Box;

/// Trait for cryptographic operations used by precompiles.
pub trait CryptoProvider: Send + Sync + 'static {
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
#[derive(Debug, Clone)]
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

/// Global crypto provider instance
static PROVIDER: OnceBox<Box<dyn CryptoProvider>> = OnceBox::new();

/// Install a custom crypto provider globally.
///
/// # Arguments
/// * `provider` - The crypto provider implementation to use
///
/// # Returns
/// `true` if the provider was installed successfully, `false` if a provider was already installed.
///
/// # Example
/// ```ignore
/// use revm_precompile::crypto_provider::{install_provider, CryptoProvider};
/// 
/// struct MyProvider;
/// impl CryptoProvider for MyProvider {
///     // ... implementation
/// }
/// 
/// if !install_provider(MyProvider) {
///     println!("Provider already installed");
/// }
/// ```
pub fn install_provider<P: CryptoProvider>(provider: P) -> bool {
    PROVIDER.set(Box::new(Box::new(provider))).is_ok()
}

/// Get the installed crypto provider, or the default if none is installed.
pub fn get_provider() -> &'static dyn CryptoProvider {
    PROVIDER.get_or_init(|| Box::new(Box::new(DefaultCryptoProvider))).as_ref()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[derive(Debug, Clone)]
    struct TestProvider;
    
    impl CryptoProvider for TestProvider {
        fn bn128_add(&self, _p1: &[u8; 64], _p2: &[u8; 64]) -> Result<[u8; 64], PrecompileError> {
            // Return a test value
            Ok([42u8; 64])
        }
    }
    
    #[test]
    fn test_default_provider() {
        // Test that the default provider works
        let p1 = [0u8; 64];
        let p2 = [0u8; 64];
        let result = get_provider().bn128_add(&p1, &p2);
        
        // The default provider should return a valid result (all zeros for zero inputs)
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_provider_installation() {
        // Test that we can create a provider instance
        let provider = TestProvider;
        let p1 = [0u8; 64];
        let p2 = [0u8; 64];
        let result = provider.bn128_add(&p1, &p2).unwrap();
        assert_eq!(result, [42u8; 64]);
    }
}