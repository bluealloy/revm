//! Cryptographic backend implementations for precompiles
//!
//! This module contains pure cryptographic implementations used by various precompiles.
//! The precompile and Ethereum specific logic (addresses, gas costs, input parsing, evm padding) remains in the parent modules.

use crate::PrecompileError;
use once_cell::race::OnceBox;
use std::boxed::Box;
use std::vec::Vec;

/// BN128 elliptic curve operations
pub mod bn128;

/// BLS12-381 elliptic curve operations  
pub mod bls12_381;

/// Blake2 compression function
pub mod blake2;

/// Hash functions (SHA-256, RIPEMD-160)
pub mod hash;

/// KZG point evaluation
#[cfg(any(feature = "c-kzg", feature = "kzg-rs"))]
pub mod kzg;

/// Modular exponentiation
pub mod modexp;

/// secp256k1 elliptic curve operations
pub mod secp256k1;

/// secp256r1 (P-256) elliptic curve operations
pub mod secp256r1;

// Import constants and types needed by the trait
use bls12_381::constants::{FP_LENGTH, G1_LENGTH, G2_LENGTH, SCALAR_LENGTH};
use bls12_381::{G1Point, G2Point, PairingPair};
use primitives::{alloy_primitives::B512, B256};

/// Trait for cryptographic operations used by precompiles.
pub trait CryptoProvider: Send + Sync + 'static {
    /// BN128 elliptic curve addition.
    fn bn128_g1_add(&self, p1_bytes: &[u8], p2_bytes: &[u8]) -> Result<[u8; 64], PrecompileError>;

    /// BN128 elliptic curve scalar multiplication.
    fn bn128_g1_mul(
        &self,
        point_bytes: &[u8],
        fr_bytes: &[u8],
    ) -> Result<[u8; 64], PrecompileError>;

    /// BN128 pairing check.
    fn bn128_pairing_check(&self, pairs: &[(&[u8], &[u8])]) -> Result<bool, PrecompileError>;

    /// BLS12-381 G1 point addition.
    fn bls12_381_g1_add(&self, a: G1Point, b: G1Point) -> Result<[u8; G1_LENGTH], PrecompileError>;

    /// BLS12-381 G2 point addition.
    fn bls12_381_g2_add(&self, a: G2Point, b: G2Point) -> Result<[u8; G2_LENGTH], PrecompileError>;

    /// BLS12-381 G1 multi-scalar multiplication.
    fn bls12_381_g1_msm(
        &self,
        points_scalars: Box<
            dyn Iterator<Item = Result<(G1Point, [u8; SCALAR_LENGTH]), PrecompileError>> + '_,
        >,
    ) -> Result<[u8; G1_LENGTH], PrecompileError>;

    /// BLS12-381 G2 multi-scalar multiplication.
    fn bls12_381_g2_msm(
        &self,
        points_scalars: Box<
            dyn Iterator<Item = Result<(G2Point, [u8; SCALAR_LENGTH]), PrecompileError>> + '_,
        >,
    ) -> Result<[u8; G2_LENGTH], PrecompileError>;

    /// BLS12-381 pairing check.
    fn bls12_381_pairing_check(&self, pairs: &[PairingPair]) -> Result<bool, PrecompileError>;

    /// BLS12-381 map field element to G1.
    fn bls12_381_fp_to_g1(
        &self,
        fp_bytes: &[u8; FP_LENGTH],
    ) -> Result<[u8; G1_LENGTH], PrecompileError>;

    /// BLS12-381 map field element to G2.
    fn bls12_381_fp2_to_g2(
        &self,
        fp2_x: &[u8; FP_LENGTH],
        fp2_y: &[u8; FP_LENGTH],
    ) -> Result<[u8; G2_LENGTH], PrecompileError>;

    /// KZG point evaluation.
    #[cfg(any(feature = "c-kzg", feature = "kzg-rs"))]
    fn verify_kzg_proof(
        &self,
        commitment: &[u8; 48],
        z: &[u8; 32],
        y: &[u8; 32],
        proof: &[u8; 48],
    ) -> bool;

    /// secp256k1 ECDSA signature recovery.
    fn ecrecover(&self, sig: &B512, recid: u8, msg: &B256) -> Option<B256>;

    /// secp256r1 (P-256) signature verification.
    fn secp256r1_verify_signature(
        &self,
        msg: &[u8; secp256r1::constants::MESSAGE_HASH_LENGTH],
        sig: &[u8; secp256r1::constants::SIGNATURE_LENGTH],
        pk: &[u8; secp256r1::constants::PUBKEY_LENGTH],
    ) -> Option<()>;

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
    /// The SHA-256 hash as 32 bytes.
    fn sha256(&self, input: &[u8]) -> [u8; 32];

    /// RIPEMD-160 hash function.
    ///
    /// Computes the RIPEMD-160 hash of the input data.
    ///
    /// # Arguments
    /// * `input` - The input data to hash
    ///
    /// # Returns
    /// The RIPEMD-160 hash as 32 bytes (20 bytes hash + 12 bytes zero padding).
    fn ripemd160(&self, input: &[u8]) -> [u8; 32];

    /// Blake2 compression function.
    fn blake2_compress(&self, rounds: usize, h: &mut [u64; 8], m: [u64; 16], t: [u64; 2], f: bool);
}

/// Default crypto provider using the existing implementations
#[derive(Debug, Clone)]
pub struct DefaultCryptoProvider;

impl CryptoProvider for DefaultCryptoProvider {
    fn bn128_g1_add(&self, p1_bytes: &[u8], p2_bytes: &[u8]) -> Result<[u8; 64], PrecompileError> {
        bn128::g1_point_add(p1_bytes, p2_bytes)
    }

    fn bn128_g1_mul(
        &self,
        point_bytes: &[u8],
        fr_bytes: &[u8],
    ) -> Result<[u8; 64], PrecompileError> {
        bn128::g1_point_mul(point_bytes, fr_bytes)
    }

    fn bn128_pairing_check(&self, pairs: &[(&[u8], &[u8])]) -> Result<bool, PrecompileError> {
        bn128::pairing_check(pairs)
    }

    fn bls12_381_g1_add(&self, a: G1Point, b: G1Point) -> Result<[u8; G1_LENGTH], PrecompileError> {
        bls12_381::p1_add_affine_bytes(a, b)
    }

    fn bls12_381_g2_add(&self, a: G2Point, b: G2Point) -> Result<[u8; G2_LENGTH], PrecompileError> {
        bls12_381::p2_add_affine_bytes(a, b)
    }

    fn bls12_381_g1_msm(
        &self,
        points_scalars: Box<
            dyn Iterator<Item = Result<(G1Point, [u8; SCALAR_LENGTH]), PrecompileError>> + '_,
        >,
    ) -> Result<[u8; G1_LENGTH], PrecompileError> {
        bls12_381::p1_msm_bytes(points_scalars)
    }

    fn bls12_381_g2_msm(
        &self,
        points_scalars: Box<
            dyn Iterator<Item = Result<(G2Point, [u8; SCALAR_LENGTH]), PrecompileError>> + '_,
        >,
    ) -> Result<[u8; G2_LENGTH], PrecompileError> {
        bls12_381::p2_msm_bytes(points_scalars)
    }

    fn bls12_381_pairing_check(&self, pairs: &[PairingPair]) -> Result<bool, PrecompileError> {
        bls12_381::pairing_check_bytes(pairs)
    }

    fn bls12_381_fp_to_g1(
        &self,
        fp_bytes: &[u8; FP_LENGTH],
    ) -> Result<[u8; G1_LENGTH], PrecompileError> {
        bls12_381::map_fp_to_g1_bytes(fp_bytes)
    }

    fn bls12_381_fp2_to_g2(
        &self,
        fp2_x: &[u8; FP_LENGTH],
        fp2_y: &[u8; FP_LENGTH],
    ) -> Result<[u8; G2_LENGTH], PrecompileError> {
        bls12_381::map_fp2_to_g2_bytes(fp2_x, fp2_y)
    }

    #[cfg(any(feature = "c-kzg", feature = "kzg-rs"))]
    fn verify_kzg_proof(
        &self,
        commitment: &[u8; 48],
        z: &[u8; 32],
        y: &[u8; 32],
        proof: &[u8; 48],
    ) -> bool {
        kzg::verify_kzg_proof(commitment, z, y, proof)
    }

    fn ecrecover(&self, sig: &B512, recid: u8, msg: &B256) -> Option<B256> {
        secp256k1::ecrecover(sig, recid, msg).ok()
    }

    fn secp256r1_verify_signature(
        &self,
        msg: &[u8; secp256r1::constants::MESSAGE_HASH_LENGTH],
        sig: &[u8; secp256r1::constants::SIGNATURE_LENGTH],
        pk: &[u8; secp256r1::constants::PUBKEY_LENGTH],
    ) -> Option<()> {
        secp256r1::verify_signature(msg, sig, pk)
    }

    fn modexp(&self, base: &[u8], exponent: &[u8], modulus: &[u8]) -> Vec<u8> {
        modexp::modexp(base, exponent, modulus)
    }

    fn sha256(&self, input: &[u8]) -> [u8; 32] {
        hash::sha256(input)
    }

    fn ripemd160(&self, input: &[u8]) -> [u8; 32] {
        hash::ripemd160(input)
    }

    fn blake2_compress(&self, rounds: usize, h: &mut [u64; 8], m: [u64; 16], t: [u64; 2], f: bool) {
        blake2::compress(rounds, h, m, t, f);
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
/// use revm_precompile::crypto::{install_provider, CryptoProvider};
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

    #[test]
    fn test_default_provider() {
        let result = get_provider().sha256(b"test");
        assert_eq!(result.len(), 32);
    }
}
