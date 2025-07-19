//! Crypto provider interface for precompiles
//!
//! This module defines a trait that abstracts cryptographic operations
//! for various precompiles, allowing different implementations to be used.

use crate::PrecompileError;
use once_cell::race::OnceBox;
use std::boxed::Box;

/// BN128 cryptographic implementations
pub mod bn128;

/// BLS12-381 cryptographic implementations
pub mod bls12_381;

/// secp256k1 cryptographic implementations
pub mod secp256k1;

/// KZG cryptographic implementations
#[cfg(any(feature = "c-kzg", feature = "kzg-rs"))]
pub mod kzg;

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

    /// BN128 elliptic curve scalar multiplication.
    ///
    /// Multiplies a point on the BN128 G1 curve by a scalar.
    ///
    /// # Arguments
    /// * `point` - Point as 64 bytes (x, y coordinates, 32 bytes each)
    /// * `scalar` - Scalar value as 32 bytes in big-endian format
    ///
    /// # Returns
    /// The product of the point and scalar as 64 bytes, or an error if inputs are invalid.
    fn bn128_mul(&self, point: &[u8; 64], scalar: &[u8; 32]) -> Result<[u8; 64], PrecompileError>;

    /// BN128 pairing check.
    ///
    /// Performs a pairing check on a list of G1 and G2 point pairs.
    ///
    /// # Arguments
    /// * `pairs` - Vector of (G1, G2) point pairs where:
    ///   - G1 points are 64 bytes each (x, y coordinates, 32 bytes each)
    ///   - G2 points are 128 bytes each (x and y coordinates in Fq2, 64 bytes each)
    ///
    /// # Returns
    /// `true` if the pairing check passes (result equals identity element), `false` otherwise.
    fn bn128_pairing(&self, pairs: &[(&[u8], &[u8])]) -> Result<bool, PrecompileError>;

    /// BLS12-381 G1 point addition.
    ///
    /// # Arguments
    /// * `a` - First G1 point as a tuple (x, y) with each component being 48 bytes
    /// * `b` - Second G1 point as a tuple (x, y) with each component being 48 bytes
    ///
    /// # Returns
    /// The sum as 96 bytes (unpadded), or an error if points are invalid.
    fn bls12_381_g1_add(
        &self,
        a: bls12_381::G1Point,
        b: bls12_381::G1Point,
    ) -> Result<[u8; 96], PrecompileError>;

    /// BLS12-381 G2 point addition.
    ///
    /// # Arguments
    /// * `a` - First G2 point as a tuple (x0, x1, y0, y1) with each component being 48 bytes
    /// * `b` - Second G2 point as a tuple (x0, x1, y0, y1) with each component being 48 bytes
    ///
    /// # Returns
    /// The sum as 192 bytes (unpadded), or an error if points are invalid.
    fn bls12_381_g2_add(
        &self,
        a: bls12_381::G2Point,
        b: bls12_381::G2Point,
    ) -> Result<[u8; 192], PrecompileError>;

    /// BLS12-381 G1 multi-scalar multiplication.
    ///
    /// # Arguments
    /// * `points_scalars` - Vector of (point, scalar) pairs where:
    ///   - Points are G1Point tuples: (x, y) with each component being 48 bytes
    ///   - Scalars are 32 bytes each
    ///
    /// # Returns
    /// The result as 96 bytes (unpadded), or an error if inputs are invalid.
    fn bls12_381_g1_msm(
        &self,
        points_scalars: &[(bls12_381::G1Point, [u8; 32])],
    ) -> Result<[u8; 96], PrecompileError>;

    /// BLS12-381 G2 multi-scalar multiplication.
    ///
    /// # Arguments
    /// * `points_scalars` - Vector of (point, scalar) pairs where:
    ///   - Points are G2Point tuples: (x0, x1, y0, y1) with each component being 48 bytes
    ///   - Scalars are 32 bytes each
    ///
    /// # Returns
    /// The result as 192 bytes (unpadded), or an error if inputs are invalid.
    fn bls12_381_g2_msm(
        &self,
        points_scalars: &[(bls12_381::G2Point, [u8; 32])],
    ) -> Result<[u8; 192], PrecompileError>;

    /// BLS12-381 pairing check.
    ///
    /// # Arguments
    /// * `pairs` - Vector of PairingPair (G1, G2) point pairs for pairing check
    ///
    /// # Returns
    /// `true` if the pairing check passes, `false` otherwise.
    fn bls12_381_pairing(&self, pairs: &[bls12_381::PairingPair]) -> Result<bool, PrecompileError>;

    /// BLS12-381 map field element to G1.
    ///
    /// # Arguments
    /// * `fp` - Field element as 48 bytes
    ///
    /// # Returns
    /// The mapped G1 point as 96 bytes (unpadded).
    fn bls12_381_map_fp_to_g1(&self, fp: &[u8; 48]) -> Result<[u8; 96], PrecompileError>;

    /// BLS12-381 map field element to G2.
    ///
    /// # Arguments
    /// * `fp2` - Fp2 element as two 48-byte field elements
    ///
    /// # Returns
    /// The mapped G2 point as 192 bytes (unpadded).
    fn bls12_381_map_fp2_to_g2(
        &self,
        fp2_x: &[u8; 48],
        fp2_y: &[u8; 48],
    ) -> Result<[u8; 192], PrecompileError>;

    /// KZG point evaluation.
    ///
    /// Verifies a KZG proof for polynomial evaluation.
    ///
    /// # Arguments
    /// * `commitment` - The KZG commitment (48 bytes)
    /// * `z` - The evaluation point (32 bytes, big-endian)
    /// * `y` - The claimed evaluation result (32 bytes, big-endian)
    /// * `proof` - The KZG proof (48 bytes)
    ///
    /// # Returns
    /// `true` if the proof is valid, `false` otherwise.
    #[cfg(any(feature = "c-kzg", feature = "kzg-rs"))]
    fn kzg_verify_proof(
        &self,
        commitment: &[u8; 48],
        z: &[u8; 32],
        y: &[u8; 32],
        proof: &[u8; 48],
    ) -> bool;
}

/// Default crypto provider using the existing implementations
#[derive(Debug, Clone)]
pub struct DefaultCryptoProvider;

impl CryptoProvider for DefaultCryptoProvider {
    fn bn128_add(&self, p1: &[u8; 64], p2: &[u8; 64]) -> Result<[u8; 64], PrecompileError> {
        self::bn128::g1_point_add(p1, p2)
    }

    fn bn128_mul(&self, point: &[u8; 64], scalar: &[u8; 32]) -> Result<[u8; 64], PrecompileError> {
        self::bn128::g1_point_mul(point, scalar)
    }

    fn bn128_pairing(&self, pairs: &[(&[u8], &[u8])]) -> Result<bool, PrecompileError> {
        self::bn128::pairing_check(pairs)
    }

    fn bls12_381_g1_add(
        &self,
        a: bls12_381::G1Point,
        b: bls12_381::G1Point,
    ) -> Result<[u8; 96], PrecompileError> {
        bls12_381::p1_add_affine_bytes(a, b)
    }

    fn bls12_381_g2_add(
        &self,
        a: bls12_381::G2Point,
        b: bls12_381::G2Point,
    ) -> Result<[u8; 192], PrecompileError> {
        bls12_381::p2_add_affine_bytes(a, b)
    }

    fn bls12_381_g1_msm(
        &self,
        points_scalars: &[(bls12_381::G1Point, [u8; 32])],
    ) -> Result<[u8; 96], PrecompileError> {
        bls12_381::p1_msm_bytes(points_scalars)
    }

    fn bls12_381_g2_msm(
        &self,
        points_scalars: &[(bls12_381::G2Point, [u8; 32])],
    ) -> Result<[u8; 192], PrecompileError> {
        bls12_381::p2_msm_bytes(points_scalars)
    }

    fn bls12_381_pairing(&self, pairs: &[bls12_381::PairingPair]) -> Result<bool, PrecompileError> {
        bls12_381::pairing_check_bytes(pairs)
    }

    fn bls12_381_map_fp_to_g1(&self, fp: &[u8; 48]) -> Result<[u8; 96], PrecompileError> {
        bls12_381::map_fp_to_g1_bytes(fp)
    }

    fn bls12_381_map_fp2_to_g2(
        &self,
        fp2_x: &[u8; 48],
        fp2_y: &[u8; 48],
    ) -> Result<[u8; 192], PrecompileError> {
        bls12_381::map_fp2_to_g2_bytes(fp2_x, fp2_y)
    }

    #[cfg(any(feature = "c-kzg", feature = "kzg-rs"))]
    fn kzg_verify_proof(
        &self,
        commitment: &[u8; 48],
        z: &[u8; 32],
        y: &[u8; 32],
        proof: &[u8; 48],
    ) -> bool {
        kzg::verify_kzg_proof(commitment, z, y, proof)
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
    PROVIDER
        .get_or_init(|| Box::new(Box::new(DefaultCryptoProvider)))
        .as_ref()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct TestProvider;

    impl CryptoProvider for TestProvider {
        fn bn128_add(&self, _p1: &[u8; 64], _p2: &[u8; 64]) -> Result<[u8; 64], PrecompileError> {
            Ok([42u8; 64])
        }

        fn bn128_mul(
            &self,
            _point: &[u8; 64],
            _scalar: &[u8; 32],
        ) -> Result<[u8; 64], PrecompileError> {
            Ok([43u8; 64])
        }

        fn bn128_pairing(&self, _pairs: &[(&[u8], &[u8])]) -> Result<bool, PrecompileError> {
            Ok(true)
        }

        fn bls12_381_g1_add(
            &self,
            _a: bls12_381::G1Point,
            _b: bls12_381::G1Point,
        ) -> Result<[u8; 96], PrecompileError> {
            Ok([44u8; 96])
        }

        fn bls12_381_g2_add(
            &self,
            _a: bls12_381::G2Point,
            _b: bls12_381::G2Point,
        ) -> Result<[u8; 192], PrecompileError> {
            Ok([45u8; 192])
        }

        fn bls12_381_g1_msm(
            &self,
            _points_scalars: &[(bls12_381::G1Point, [u8; 32])],
        ) -> Result<[u8; 96], PrecompileError> {
            Ok([46u8; 96])
        }

        fn bls12_381_g2_msm(
            &self,
            _points_scalars: &[(bls12_381::G2Point, [u8; 32])],
        ) -> Result<[u8; 192], PrecompileError> {
            Ok([47u8; 192])
        }

        fn bls12_381_pairing(
            &self,
            _pairs: &[bls12_381::PairingPair],
        ) -> Result<bool, PrecompileError> {
            Ok(true)
        }

        fn bls12_381_map_fp_to_g1(&self, _fp: &[u8; 48]) -> Result<[u8; 96], PrecompileError> {
            Ok([48u8; 96])
        }

        fn bls12_381_map_fp2_to_g2(
            &self,
            _fp2_x: &[u8; 48],
            _fp2_y: &[u8; 48],
        ) -> Result<[u8; 192], PrecompileError> {
            Ok([49u8; 192])
        }

        fn kzg_verify_proof(
            &self,
            _commitment: &[u8; 48],
            _z: &[u8; 32],
            _y: &[u8; 32],
            _proof: &[u8; 48],
        ) -> bool {
            true
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
