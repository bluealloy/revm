//! Interface for the precompiles. It contains the precompile result type,
//! the precompile output type, and the precompile error type.
use core::fmt::{self, Debug};
use primitives::Bytes;

extern crate alloc;
use alloc::{boxed::Box, string::String, vec::Vec};

use crate::bls12_381::{G1Point, G2Point};

/// A precompile operation result type
///
/// Returns either `Ok((gas_used, return_bytes))` or `Err(error)`.
pub type PrecompileResult = Result<PrecompileOutput, PrecompileError>;

/// Precompile execution output
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PrecompileOutput {
    /// Gas used by the precompile
    pub gas_used: u64,
    /// Output bytes
    pub bytes: Bytes,
    /// Whether the precompile reverted
    pub reverted: bool,
}

impl PrecompileOutput {
    /// Returns new precompile output with the given gas used and output bytes.
    pub fn new(gas_used: u64, bytes: Bytes) -> Self {
        Self {
            gas_used,
            bytes,
            reverted: false,
        }
    }

    /// Returns new precompile revert with the given gas used and output bytes.
    pub fn new_reverted(gas_used: u64, bytes: Bytes) -> Self {
        Self {
            gas_used,
            bytes,
            reverted: true,
        }
    }

    /// Flips [`Self::reverted`] to `true`.
    pub fn reverted(mut self) -> Self {
        self.reverted = true;
        self
    }
}

/// Crypto operations trait for precompiles.
pub trait Crypto: Send + Sync + Debug {
    /// Clone box type
    fn clone_box(&self) -> Box<dyn Crypto>;

    /// Compute SHA-256 hash
    fn sha256(&self, input: &[u8]) -> [u8; 32];

    /// Compute RIPEMD-160 hash
    fn ripemd160(&self, input: &[u8]) -> [u8; 32];

    /// BN128 elliptic curve addition.
    fn bn128_g1_add(&self, p1: &[u8], p2: &[u8]) -> Result<[u8; 64], PrecompileError>;

    /// BN128 elliptic curve scalar multiplication.
    fn bn128_g1_mul(&self, point: &[u8], scalar: &[u8]) -> Result<[u8; 64], PrecompileError>;

    /// BN128 pairing check.
    fn bn128_pairing_check(&self, pairs: &[(&[u8], &[u8])]) -> Result<bool, PrecompileError>;

    /// secp256k1 ECDSA signature recovery.
    fn secp256k1_ecrecover(
        &self,
        sig: &[u8; 64],
        recid: u8,
        msg: &[u8; 32],
    ) -> Result<[u8; 32], PrecompileError>;

    /// Modular exponentiation.
    fn modexp(&self, base: &[u8], exp: &[u8], modulus: &[u8]) -> Result<Vec<u8>, PrecompileError>;

    /// Blake2 compression function.
    fn blake2_compress(
        &self,
        rounds: u32,
        h: &mut [u64; 8],
        m: [u64; 16],
        t: [u64; 2],
        f: bool,
    ) -> Result<(), PrecompileError>;

    /// secp256r1 (P-256) signature verification.
    fn secp256r1_verify_signature(
        &self,
        msg: &[u8; 32],
        sig: &[u8; 64],
        pk: &[u8; 64],
    ) -> Result<bool, PrecompileError>;

    /// KZG point evaluation.
    #[cfg(any(feature = "c-kzg", feature = "kzg-rs"))]
    fn verify_kzg_proof(
        &self,
        z: &[u8; 32],
        y: &[u8; 32],
        commitment: &[u8; 48],
        proof: &[u8; 48],
    ) -> Result<(), PrecompileError>;

    /// BLS12-381 G1 addition (returns 96-byte unpadded G1 point)
    fn bls12_381_g1_add(&self, a: G1Point, b: G1Point) -> Result<[u8; 96], PrecompileError>;

    /// BLS12-381 G1 multi-scalar multiplication (returns 96-byte unpadded G1 point)
    fn bls12_381_g1_msm<'a>(
        &self,
        pairs: Box<dyn Iterator<Item = Result<(G1Point, [u8; 32]), PrecompileError>> + 'a>,
    ) -> Result<[u8; 96], PrecompileError>;

    /// BLS12-381 G2 addition (returns 192-byte unpadded G2 point)
    fn bls12_381_g2_add(&self, a: G2Point, b: G2Point) -> Result<[u8; 192], PrecompileError>;

    /// BLS12-381 G2 multi-scalar multiplication (returns 192-byte unpadded G2 point)
    fn bls12_381_g2_msm<'a>(
        &self,
        pairs: Box<dyn Iterator<Item = Result<(G2Point, [u8; 32]), PrecompileError>> + 'a>,
    ) -> Result<[u8; 192], PrecompileError>;

    /// BLS12-381 pairing check.
    fn bls12_381_pairing_check(
        &self,
        pairs: &[(G1Point, G2Point)],
    ) -> Result<bool, PrecompileError>;

    /// BLS12-381 map field element to G1.
    fn bls12_381_fp_to_g1(&self, fp: &[u8; 48]) -> Result<[u8; 96], PrecompileError>;

    /// BLS12-381 map field element to G2.
    fn bls12_381_fp2_to_g2(&self, fp2: ([u8; 48], [u8; 48])) -> Result<[u8; 192], PrecompileError>;
}

/// Precompile function type. Takes input, gas limit, and crypto implementation and returns precompile result.
pub type PrecompileFn = fn(&[u8], u64, &dyn Crypto) -> PrecompileResult;

/// Precompile error type.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PrecompileError {
    /// out of gas is the main error. Others are here just for completeness
    OutOfGas,
    /// Blake2 errors
    Blake2WrongLength,
    /// Blake2 wrong final indicator flag
    Blake2WrongFinalIndicatorFlag,
    /// Modexp errors
    ModexpExpOverflow,
    /// Modexp base overflow
    ModexpBaseOverflow,
    /// Modexp mod overflow
    ModexpModOverflow,
    /// Modexp limit all input sizes.
    ModexpEip7823LimitSize,
    /// Bn128 errors
    Bn128FieldPointNotAMember,
    /// Bn128 affine g failed to create
    Bn128AffineGFailedToCreate,
    /// Bn128 pair length
    Bn128PairLength,
    // Blob errors
    /// The input length is not exactly 192 bytes
    BlobInvalidInputLength,
    /// The commitment does not match the versioned hash
    BlobMismatchedVersion,
    /// The proof verification failed
    BlobVerifyKzgProofFailed,
    /// Fatal error with a custom error message
    Fatal(String),
    /// Catch-all variant for other errors
    Other(String),
}

impl PrecompileError {
    /// Returns another error with the given message.
    pub fn other(err: impl Into<String>) -> Self {
        Self::Other(err.into())
    }

    /// Returns `true` if the error is out of gas.
    pub fn is_oog(&self) -> bool {
        matches!(self, Self::OutOfGas)
    }
}

impl core::error::Error for PrecompileError {}

impl fmt::Display for PrecompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::OutOfGas => "out of gas",
            Self::Blake2WrongLength => "wrong input length for blake2",
            Self::Blake2WrongFinalIndicatorFlag => "wrong final indicator flag for blake2",
            Self::ModexpExpOverflow => "modexp exp overflow",
            Self::ModexpBaseOverflow => "modexp base overflow",
            Self::ModexpModOverflow => "modexp mod overflow",
            Self::ModexpEip7823LimitSize => "Modexp limit all input sizes.",
            Self::Bn128FieldPointNotAMember => "field point not a member of bn128 curve",
            Self::Bn128AffineGFailedToCreate => "failed to create affine g point for bn128 curve",
            Self::Bn128PairLength => "bn128 invalid pair length",
            Self::BlobInvalidInputLength => "invalid blob input length",
            Self::BlobMismatchedVersion => "mismatched blob version",
            Self::BlobVerifyKzgProofFailed => "verifying blob kzg proof failed",
            Self::Fatal(s) => s,
            Self::Other(s) => s,
        };
        f.write_str(s)
    }
}

/// Default implementation of the Crypto trait using the existing crypto libraries.
#[derive(Clone, Debug)]
pub struct DefaultCrypto;

impl Crypto for DefaultCrypto {
    fn clone_box(&self) -> Box<dyn Crypto> {
        Box::new(self.clone())
    }

    fn sha256(&self, input: &[u8]) -> [u8; 32] {
        use sha2::Digest;
        let output = sha2::Sha256::digest(input);
        output.into()
    }

    fn ripemd160(&self, input: &[u8]) -> [u8; 32] {
        use ripemd::Digest;
        let mut hasher = ripemd::Ripemd160::new();
        hasher.update(input);

        let mut output = [0u8; 32];
        hasher.finalize_into((&mut output[12..]).into());
        output
    }

    fn bn128_g1_add(&self, p1: &[u8], p2: &[u8]) -> Result<[u8; 64], PrecompileError> {
        crate::bn128::crypto_backend::g1_point_add(p1, p2)
    }

    fn bn128_g1_mul(&self, point: &[u8], scalar: &[u8]) -> Result<[u8; 64], PrecompileError> {
        crate::bn128::crypto_backend::g1_point_mul(point, scalar)
    }

    fn bn128_pairing_check(&self, pairs: &[(&[u8], &[u8])]) -> Result<bool, PrecompileError> {
        crate::bn128::crypto_backend::pairing_check(pairs)
    }

    fn secp256k1_ecrecover(
        &self,
        sig: &[u8; 64],
        recid: u8,
        msg: &[u8; 32],
    ) -> Result<[u8; 32], PrecompileError> {
        crate::secp256k1::ecrecover_bytes(*sig, recid, *msg)
            .ok_or_else(|| PrecompileError::other("ecrecover failed"))
    }

    fn modexp(&self, base: &[u8], exp: &[u8], modulus: &[u8]) -> Result<Vec<u8>, PrecompileError> {
        Ok(crate::modexp::modexp(base, exp, modulus))
    }

    fn blake2_compress(
        &self,
        rounds: u32,
        h: &mut [u64; 8],
        m: [u64; 16],
        t: [u64; 2],
        f: bool,
    ) -> Result<(), PrecompileError> {
        crate::blake2::algo::compress(rounds as usize, h, m, t, f);
        Ok(())
    }

    fn secp256r1_verify_signature(
        &self,
        msg: &[u8; 32],
        sig: &[u8; 64],
        pk: &[u8; 64],
    ) -> Result<bool, PrecompileError> {
        Ok(crate::secp256r1::verify_signature(*msg, *sig, *pk).is_some())
    }

    #[cfg(any(feature = "c-kzg", feature = "kzg-rs"))]
    fn verify_kzg_proof(
        &self,
        z: &[u8; 32],
        y: &[u8; 32],
        commitment: &[u8; 48],
        proof: &[u8; 48],
    ) -> Result<(), PrecompileError> {
        if !crate::kzg_point_evaluation::verify_kzg_proof(commitment, z, y, proof) {
            return Err(PrecompileError::BlobVerifyKzgProofFailed);
        }

        Ok(())
    }

    fn bls12_381_g1_add(&self, a: G1Point, b: G1Point) -> Result<[u8; 96], PrecompileError> {
        crate::bls12_381::crypto_backend::p1_add_affine_bytes(a, b)
    }

    fn bls12_381_g1_msm<'a>(
        &self,
        pairs: Box<dyn Iterator<Item = Result<(G1Point, [u8; 32]), PrecompileError>> + 'a>,
    ) -> Result<[u8; 96], PrecompileError> {
        crate::bls12_381::crypto_backend::p1_msm_bytes(pairs)
    }

    fn bls12_381_g2_add(&self, a: G2Point, b: G2Point) -> Result<[u8; 192], PrecompileError> {
        crate::bls12_381::crypto_backend::p2_add_affine_bytes(a, b)
    }

    fn bls12_381_g2_msm<'a>(
        &self,
        pairs: Box<dyn Iterator<Item = Result<(G2Point, [u8; 32]), PrecompileError>> + 'a>,
    ) -> Result<[u8; 192], PrecompileError> {
        crate::bls12_381::crypto_backend::p2_msm_bytes(pairs)
    }

    fn bls12_381_pairing_check(
        &self,
        pairs: &[(G1Point, G2Point)],
    ) -> Result<bool, PrecompileError> {
        crate::bls12_381::crypto_backend::pairing_check_bytes(pairs)
    }

    fn bls12_381_fp_to_g1(&self, fp: &[u8; 48]) -> Result<[u8; 96], PrecompileError> {
        crate::bls12_381::crypto_backend::map_fp_to_g1_bytes(fp)
    }

    fn bls12_381_fp2_to_g2(&self, fp2: ([u8; 48], [u8; 48])) -> Result<[u8; 192], PrecompileError> {
        crate::bls12_381::crypto_backend::map_fp2_to_g2_bytes(&fp2.0, &fp2.1)
    }
}
