//! Crypto provider interface for precompiles
//!
//! This module defines a trait that abstracts cryptographic operations
//! for various precompiles, allowing different implementations to be used.

use crate::PrecompileError;
use once_cell::race::OnceBox;
use std::boxed::Box;
use std::vec::Vec;

/// BN128 cryptographic implementations
pub mod bn128;

/// BLS12-381 cryptographic implementations
pub mod bls12_381;

/// secp256k1 cryptographic implementations
pub mod secp256k1;

/// KZG cryptographic implementations
#[cfg(any(feature = "c-kzg", feature = "kzg-rs"))]
pub mod kzg;

/// Modexp cryptographic implementations
pub mod modexp;

/// Hash cryptographic implementations
pub mod hash;

/// Blake2 cryptographic implementations
pub mod blake2;

/// secp256r1 cryptographic implementations
pub mod secp256r1;

/// Trait for cryptographic operations used by precompiles.
pub trait CryptoProvider: Send + Sync + 'static {
    /// BN128 elliptic curve addition.
    ///
    /// Takes two points on the BN128 G1 curve and returns their sum.
    ///
    /// # Arguments
    /// * `p1` - First point as bn128::G1_LENGTH bytes (x, y coordinates, 32 bytes each)
    /// * `p2` - Second point as bn128::G1_LENGTH bytes (x, y coordinates, 32 bytes each)
    ///
    /// # Returns
    /// The sum of the two points as bn128::G1_LENGTH bytes, or an error if the points are invalid.
    fn bn128_add(
        &self,
        p1: &[u8; bn128::G1_LENGTH],
        p2: &[u8; bn128::G1_LENGTH],
    ) -> Result<[u8; bn128::G1_LENGTH], PrecompileError>;

    /// BN128 elliptic curve scalar multiplication.
    ///
    /// Multiplies a point on the BN128 G1 curve by a scalar.
    ///
    /// # Arguments
    /// * `point` - Point as bn128::G1_LENGTH bytes (x, y coordinates, 32 bytes each)
    /// * `scalar` - Scalar value as bn128::SCALAR_LENGTH bytes in big-endian format
    ///
    /// # Returns
    /// The product of the point and scalar as bn128::G1_LENGTH bytes, or an error if inputs are invalid.
    fn bn128_mul(
        &self,
        point: &[u8; bn128::G1_LENGTH],
        scalar: &[u8; bn128::SCALAR_LENGTH],
    ) -> Result<[u8; bn128::G1_LENGTH], PrecompileError>;

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
    /// * `a` - First G1 point as a tuple (x, y) with each component being bls12_381::FP_LENGTH bytes
    /// * `b` - Second G1 point as a tuple (x, y) with each component being bls12_381::FP_LENGTH bytes
    ///
    /// # Returns
    /// The sum as bls12_381::G1_LENGTH bytes (unpadded), or an error if points are invalid.
    fn bls12_381_g1_add(
        &self,
        a: bls12_381::G1Point,
        b: bls12_381::G1Point,
    ) -> Result<[u8; bls12_381::G1_LENGTH], PrecompileError>;

    /// BLS12-381 G2 point addition.
    ///
    /// # Arguments
    /// * `a` - First G2 point as a tuple (x0, x1, y0, y1) with each component being bls12_381::FP_LENGTH bytes
    /// * `b` - Second G2 point as a tuple (x0, x1, y0, y1) with each component being bls12_381::FP_LENGTH bytes
    ///
    /// # Returns
    /// The sum as bls12_381::G2_LENGTH bytes (unpadded), or an error if points are invalid.
    fn bls12_381_g2_add(
        &self,
        a: bls12_381::G2Point,
        b: bls12_381::G2Point,
    ) -> Result<[u8; bls12_381::G2_LENGTH], PrecompileError>;

    /// BLS12-381 G1 multi-scalar multiplication.
    ///
    /// # Arguments
    /// * `points_scalars` - Iterator of (point, scalar) pairs where:
    ///   - Points are G1Point tuples: (x, y) with each component being bls12_381::FP_LENGTH bytes
    ///   - Scalars are bls12_381::SCALAR_LENGTH bytes each
    ///
    /// # Returns
    /// The result as bls12_381::G1_LENGTH bytes (unpadded), or an error if inputs are invalid.
    fn bls12_381_g1_msm(
        &self,
        points_scalars: Box<
            dyn Iterator<
                    Item = Result<
                        (bls12_381::G1Point, [u8; bls12_381::SCALAR_LENGTH]),
                        PrecompileError,
                    >,
                > + '_,
        >,
    ) -> Result<[u8; bls12_381::G1_LENGTH], PrecompileError>;

    /// BLS12-381 G2 multi-scalar multiplication.
    ///
    /// # Arguments
    /// * `points_scalars` - Iterator of (point, scalar) pairs where:
    ///   - Points are G2Point tuples: (x0, x1, y0, y1) with each component being bls12_381::FP_LENGTH bytes
    ///   - Scalars are bls12_381::SCALAR_LENGTH bytes each
    ///
    /// # Returns
    /// The result as bls12_381::G2_LENGTH bytes (unpadded), or an error if inputs are invalid.
    fn bls12_381_g2_msm(
        &self,
        points_scalars: Box<
            dyn Iterator<
                    Item = Result<
                        (bls12_381::G2Point, [u8; bls12_381::SCALAR_LENGTH]),
                        PrecompileError,
                    >,
                > + '_,
        >,
    ) -> Result<[u8; bls12_381::G2_LENGTH], PrecompileError>;

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
    /// * `fp` - Field element as bls12_381::FP_LENGTH bytes
    ///
    /// # Returns
    /// The mapped G1 point as bls12_381::G1_LENGTH bytes (unpadded).
    fn bls12_381_map_fp_to_g1(
        &self,
        fp: &[u8; bls12_381::FP_LENGTH],
    ) -> Result<[u8; bls12_381::G1_LENGTH], PrecompileError>;

    /// BLS12-381 map field element to G2.
    ///
    /// # Arguments
    /// * `fp2` - Fp2 element as two bls12_381::FP_LENGTH-byte field elements
    ///
    /// # Returns
    /// The mapped G2 point as bls12_381::G2_LENGTH bytes (unpadded).
    fn bls12_381_map_fp2_to_g2(
        &self,
        fp2_x: &[u8; bls12_381::FP_LENGTH],
        fp2_y: &[u8; bls12_381::FP_LENGTH],
    ) -> Result<[u8; bls12_381::G2_LENGTH], PrecompileError>;

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

    /// secp256k1 ECDSA signature recovery.
    ///
    /// Recovers the Ethereum address from an ECDSA signature.
    ///
    /// # Arguments
    /// * `sig` - The signature (secp256k1::SIGNATURE_LENGTH bytes: r || s)
    /// * `recid` - The recovery ID (0 or 1)
    /// * `msg` - The message hash (secp256k1::MESSAGE_HASH_LENGTH bytes)
    ///
    /// # Returns
    /// The recovered address as secp256k1::MESSAGE_HASH_LENGTH bytes (first 12 bytes are zero, last 20 bytes are the address), or None if recovery fails.
    fn secp256k1_ecrecover(
        &self,
        sig: &[u8; secp256k1::SIGNATURE_LENGTH],
        recid: u8,
        msg: &[u8; secp256k1::MESSAGE_HASH_LENGTH],
    ) -> Option<[u8; secp256k1::MESSAGE_HASH_LENGTH]>;

    /// secp256r1 (P-256) signature verification.
    ///
    /// Verifies a secp256r1 signature.
    ///
    /// # Arguments
    /// * `msg` - The message hash (secp256r1::MESSAGE_HASH_LENGTH bytes)
    /// * `sig` - The signature (secp256r1::SIGNATURE_LENGTH bytes: r || s)
    /// * `pk` - The uncompressed public key (secp256r1::PUBKEY_LENGTH bytes: 0x04 || x || y)
    ///
    /// # Returns
    /// `true` if the signature is valid, `false` otherwise.
    fn secp256r1_verify(
        &self,
        msg: &[u8; secp256r1::MESSAGE_HASH_LENGTH],
        sig: &[u8; secp256r1::SIGNATURE_LENGTH],
        pk: &[u8; secp256r1::PUBKEY_LENGTH],
    ) -> bool;

    /// Modular exponentiation.
    ///
    /// Computes base^exponent mod modulus.
    ///
    /// # Arguments
    /// * `base` - The base value
    /// * `exponent` - The exponent value
    /// * `modulus` - The modulus value
    ///
    /// # Returns
    /// The result of the modular exponentiation.
    fn modexp(&self, base: &[u8], exponent: &[u8], modulus: &[u8]) -> Vec<u8>;

    /// SHA-256 hash function.
    ///
    /// Computes the SHA-256 hash of the input data.
    ///
    /// # Arguments
    /// * `input` - The input data to hash
    ///
    /// # Returns
    /// The SHA-256 hash as hash::SHA256_LENGTH bytes.
    fn sha256(&self, input: &[u8]) -> [u8; hash::SHA256_LENGTH];

    /// RIPEMD-160 hash function.
    ///
    /// Computes the RIPEMD-160 hash of the input data.
    ///
    /// # Arguments
    /// * `input` - The input data to hash
    ///
    /// # Returns
    /// The RIPEMD-160 hash as hash::RIPEMD160_LENGTH bytes.
    fn ripemd160(&self, input: &[u8]) -> [u8; hash::RIPEMD160_LENGTH];

    /// Blake2 compression function.
    ///
    /// Performs the Blake2b compression function F.
    ///
    /// # Arguments
    /// * `rounds` - Number of rounds to perform
    /// * `h` - State vector (blake2::STATE_LENGTH u64 values)
    /// * `m` - Message block (blake2::MESSAGE_LENGTH bytes)
    /// * `t` - Offset counter (2 u64 values)
    /// * `f` - Final block indicator flag
    ///
    /// # Returns
    /// The compressed state vector (blake2::STATE_LENGTH u64 values).
    fn blake2_compress(
        &self,
        rounds: usize,
        h: [u64; blake2::STATE_LENGTH],
        m: &[u8; blake2::MESSAGE_LENGTH],
        t: [u64; 2],
        f: bool,
    ) -> [u64; blake2::STATE_LENGTH];
}

/// Default crypto provider using the existing implementations
#[derive(Debug, Clone)]
pub struct DefaultCryptoProvider;

impl CryptoProvider for DefaultCryptoProvider {
    fn bn128_add(
        &self,
        p1: &[u8; bn128::G1_LENGTH],
        p2: &[u8; bn128::G1_LENGTH],
    ) -> Result<[u8; bn128::G1_LENGTH], PrecompileError> {
        bn128::g1_point_add(p1, p2)
    }

    fn bn128_mul(
        &self,
        point: &[u8; bn128::G1_LENGTH],
        scalar: &[u8; bn128::SCALAR_LENGTH],
    ) -> Result<[u8; bn128::G1_LENGTH], PrecompileError> {
        bn128::g1_point_mul(point, scalar)
    }

    fn bn128_pairing(&self, pairs: &[(&[u8], &[u8])]) -> Result<bool, PrecompileError> {
        bn128::pairing_check(pairs)
    }

    fn bls12_381_g1_add(
        &self,
        a: bls12_381::G1Point,
        b: bls12_381::G1Point,
    ) -> Result<[u8; bls12_381::G1_LENGTH], PrecompileError> {
        bls12_381::p1_add_affine_bytes(a, b)
    }

    fn bls12_381_g2_add(
        &self,
        a: bls12_381::G2Point,
        b: bls12_381::G2Point,
    ) -> Result<[u8; bls12_381::G2_LENGTH], PrecompileError> {
        bls12_381::p2_add_affine_bytes(a, b)
    }

    fn bls12_381_g1_msm(
        &self,
        points_scalars: Box<
            dyn Iterator<
                    Item = Result<
                        (bls12_381::G1Point, [u8; bls12_381::SCALAR_LENGTH]),
                        PrecompileError,
                    >,
                > + '_,
        >,
    ) -> Result<[u8; bls12_381::G1_LENGTH], PrecompileError> {
        bls12_381::p1_msm_bytes(points_scalars)
    }

    fn bls12_381_g2_msm(
        &self,
        points_scalars: Box<
            dyn Iterator<
                    Item = Result<
                        (bls12_381::G2Point, [u8; bls12_381::SCALAR_LENGTH]),
                        PrecompileError,
                    >,
                > + '_,
        >,
    ) -> Result<[u8; bls12_381::G2_LENGTH], PrecompileError> {
        bls12_381::p2_msm_bytes(points_scalars)
    }

    fn bls12_381_pairing(&self, pairs: &[bls12_381::PairingPair]) -> Result<bool, PrecompileError> {
        bls12_381::pairing_check_bytes(pairs)
    }

    fn bls12_381_map_fp_to_g1(
        &self,
        fp: &[u8; bls12_381::FP_LENGTH],
    ) -> Result<[u8; bls12_381::G1_LENGTH], PrecompileError> {
        bls12_381::map_fp_to_g1_bytes(fp)
    }

    fn bls12_381_map_fp2_to_g2(
        &self,
        fp2_x: &[u8; bls12_381::FP_LENGTH],
        fp2_y: &[u8; bls12_381::FP_LENGTH],
    ) -> Result<[u8; bls12_381::G2_LENGTH], PrecompileError> {
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

    fn secp256k1_ecrecover(
        &self,
        sig: &[u8; secp256k1::SIGNATURE_LENGTH],
        recid: u8,
        msg: &[u8; secp256k1::MESSAGE_HASH_LENGTH],
    ) -> Option<[u8; secp256k1::MESSAGE_HASH_LENGTH]> {
        use primitives::{alloy_primitives::B512, B256};
        let sig = B512::from_slice(sig);
        let msg = B256::from_slice(msg);

        match secp256k1::ecrecover(&sig, recid, &msg) {
            Ok(address) => Some(address.0),
            Err(_) => None,
        }
    }

    fn secp256r1_verify(
        &self,
        msg: &[u8; secp256r1::MESSAGE_HASH_LENGTH],
        sig: &[u8; secp256r1::SIGNATURE_LENGTH],
        pk: &[u8; secp256r1::PUBKEY_LENGTH],
    ) -> bool {
        secp256r1::verify(msg, sig, pk)
    }

    fn modexp(&self, base: &[u8], exponent: &[u8], modulus: &[u8]) -> Vec<u8> {
        modexp::modexp(base, exponent, modulus)
    }

    fn sha256(&self, input: &[u8]) -> [u8; hash::SHA256_LENGTH] {
        hash::sha256(input)
    }

    fn ripemd160(&self, input: &[u8]) -> [u8; hash::RIPEMD160_LENGTH] {
        hash::ripemd160(input)
    }

    fn blake2_compress(
        &self,
        rounds: usize,
        h: [u64; blake2::STATE_LENGTH],
        m: &[u8; blake2::MESSAGE_LENGTH],
        t: [u64; 2],
        f: bool,
    ) -> [u64; blake2::STATE_LENGTH] {
        blake2::compress(rounds, h, m, t, f)
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
        ) -> Result<[u8; bls12_381::G1_LENGTH], PrecompileError> {
            Ok([44u8; bls12_381::G1_LENGTH])
        }

        fn bls12_381_g2_add(
            &self,
            _a: bls12_381::G2Point,
            _b: bls12_381::G2Point,
        ) -> Result<[u8; bls12_381::G2_LENGTH], PrecompileError> {
            Ok([45u8; bls12_381::G2_LENGTH])
        }

        fn bls12_381_g1_msm(
            &self,
            _points_scalars: Box<
                dyn Iterator<
                        Item = Result<
                            (bls12_381::G1Point, [u8; bls12_381::SCALAR_LENGTH]),
                            PrecompileError,
                        >,
                    > + '_,
            >,
        ) -> Result<[u8; bls12_381::G1_LENGTH], PrecompileError> {
            Ok([46u8; bls12_381::G1_LENGTH])
        }

        fn bls12_381_g2_msm(
            &self,
            _points_scalars: Box<
                dyn Iterator<
                        Item = Result<
                            (bls12_381::G2Point, [u8; bls12_381::SCALAR_LENGTH]),
                            PrecompileError,
                        >,
                    > + '_,
            >,
        ) -> Result<[u8; bls12_381::G2_LENGTH], PrecompileError> {
            Ok([47u8; bls12_381::G2_LENGTH])
        }

        fn bls12_381_pairing(
            &self,
            _pairs: &[bls12_381::PairingPair],
        ) -> Result<bool, PrecompileError> {
            Ok(true)
        }

        fn bls12_381_map_fp_to_g1(
            &self,
            _fp: &[u8; bls12_381::FP_LENGTH],
        ) -> Result<[u8; bls12_381::G1_LENGTH], PrecompileError> {
            Ok([48u8; bls12_381::G1_LENGTH])
        }

        fn bls12_381_map_fp2_to_g2(
            &self,
            _fp2_x: &[u8; bls12_381::FP_LENGTH],
            _fp2_y: &[u8; bls12_381::FP_LENGTH],
        ) -> Result<[u8; bls12_381::G2_LENGTH], PrecompileError> {
            Ok([49u8; bls12_381::G2_LENGTH])
        }

        #[cfg(any(feature = "c-kzg", feature = "kzg-rs"))]
        fn kzg_verify_proof(
            &self,
            _commitment: &[u8; 48],
            _z: &[u8; 32],
            _y: &[u8; 32],
            _proof: &[u8; 48],
        ) -> bool {
            true
        }

        fn secp256k1_ecrecover(
            &self,
            _sig: &[u8; 64],
            _recid: u8,
            _msg: &[u8; 32],
        ) -> Option<[u8; 32]> {
            Some([50u8; 32])
        }

        fn secp256r1_verify(
            &self,
            _msg: &[u8; secp256r1::MESSAGE_HASH_LENGTH],
            _sig: &[u8; secp256r1::SIGNATURE_LENGTH],
            _pk: &[u8; secp256r1::PUBKEY_LENGTH],
        ) -> bool {
            true
        }

        fn modexp(&self, _base: &[u8], _exponent: &[u8], _modulus: &[u8]) -> Vec<u8> {
            vec![51u8; 32]
        }

        fn sha256(&self, _input: &[u8]) -> [u8; 32] {
            [52u8; 32]
        }

        fn ripemd160(&self, _input: &[u8]) -> [u8; 20] {
            [53u8; 20]
        }

        fn blake2_compress(
            &self,
            _rounds: usize,
            _h: [u64; 8],
            _m: &[u8; 128],
            _t: [u64; 2],
            _f: bool,
        ) -> [u64; 8] {
            [54u64; 8]
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
