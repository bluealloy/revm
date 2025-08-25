//! Interface for the precompiles. It contains the precompile result type,
//! the precompile output type, and the precompile error type.
use core::fmt::{self, Debug};
use primitives::{Bytes, OnceLock};
use std::{boxed::Box, vec::Vec};

use crate::bls12_381::{G1Point, G1PointScalar, G2Point, G2PointScalar};

/// Global crypto provider instance
static CRYPTO: OnceLock<Box<dyn Crypto>> = OnceLock::new();

/// Install a custom crypto provider globally.
pub fn install_crypto<C: Crypto + 'static>(crypto: C) -> bool {
    if CRYPTO.get().is_some() {
        return false;
    }

    CRYPTO.get_or_init(|| Box::new(crypto));
    true
}

/// Get the installed crypto provider, or the default if none is installed.
pub fn crypto() -> &'static dyn Crypto {
    CRYPTO.get_or_init(|| Box::new(DefaultCrypto)).as_ref()
}

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
    /// Compute SHA-256 hash
    #[inline]
    fn sha256(&self, input: &[u8]) -> [u8; 32] {
        use sha2::Digest;
        let output = sha2::Sha256::digest(input);
        output.into()
    }

    /// Compute RIPEMD-160 hash
    #[inline]
    fn ripemd160(&self, input: &[u8]) -> [u8; 32] {
        use ripemd::Digest;
        let mut hasher = ripemd::Ripemd160::new();
        hasher.update(input);

        let mut output = [0u8; 32];
        hasher.finalize_into((&mut output[12..]).into());
        output
    }

    /// BN254 elliptic curve addition.
    #[inline]
    fn bn254_g1_add(&self, p1: &[u8], p2: &[u8]) -> Result<[u8; 64], PrecompileError> {
        crate::bn254::crypto_backend::g1_point_add(p1, p2)
    }

    /// BN254 elliptic curve scalar multiplication.
    #[inline]
    fn bn254_g1_mul(&self, point: &[u8], scalar: &[u8]) -> Result<[u8; 64], PrecompileError> {
        crate::bn254::crypto_backend::g1_point_mul(point, scalar)
    }

    /// BN254 pairing check.
    #[inline]
    fn bn254_pairing_check(&self, pairs: &[(&[u8], &[u8])]) -> Result<bool, PrecompileError> {
        crate::bn254::crypto_backend::pairing_check(pairs)
    }

    /// secp256k1 ECDSA signature recovery.
    #[inline]
    fn secp256k1_ecrecover(
        &self,
        sig: &[u8; 64],
        recid: u8,
        msg: &[u8; 32],
    ) -> Result<[u8; 32], PrecompileError> {
        crate::secp256k1::ecrecover_bytes(*sig, recid, *msg)
            .ok_or(PrecompileError::Secp256k1RecoverFailed)
    }

    /// Modular exponentiation.
    #[inline]
    fn modexp(&self, base: &[u8], exp: &[u8], modulus: &[u8]) -> Result<Vec<u8>, PrecompileError> {
        Ok(crate::modexp::modexp(base, exp, modulus))
    }

    /// Blake2 compression function.
    #[inline]
    fn blake2_compress(&self, rounds: u32, h: &mut [u64; 8], m: [u64; 16], t: [u64; 2], f: bool) {
        crate::blake2::algo::compress(rounds as usize, h, m, t, f);
    }

    /// secp256r1 (P-256) signature verification.
    #[inline]
    fn secp256r1_verify_signature(&self, msg: &[u8; 32], sig: &[u8; 64], pk: &[u8; 64]) -> bool {
        crate::secp256r1::verify_signature(*msg, *sig, *pk).is_some()
    }

    /// KZG point evaluation.
    #[inline]
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

    /// BLS12-381 G1 addition (returns 96-byte unpadded G1 point)
    fn bls12_381_g1_add(&self, a: G1Point, b: G1Point) -> Result<[u8; 96], PrecompileError> {
        crate::bls12_381::crypto_backend::p1_add_affine_bytes(a, b)
    }

    /// BLS12-381 G1 multi-scalar multiplication (returns 96-byte unpadded G1 point)
    fn bls12_381_g1_msm(
        &self,
        pairs: &mut dyn Iterator<Item = Result<G1PointScalar, PrecompileError>>,
    ) -> Result<[u8; 96], PrecompileError> {
        crate::bls12_381::crypto_backend::p1_msm_bytes(pairs)
    }

    /// BLS12-381 G2 addition (returns 192-byte unpadded G2 point)
    fn bls12_381_g2_add(&self, a: G2Point, b: G2Point) -> Result<[u8; 192], PrecompileError> {
        crate::bls12_381::crypto_backend::p2_add_affine_bytes(a, b)
    }

    /// BLS12-381 G2 multi-scalar multiplication (returns 192-byte unpadded G2 point)
    fn bls12_381_g2_msm(
        &self,
        pairs: &mut dyn Iterator<Item = Result<G2PointScalar, PrecompileError>>,
    ) -> Result<[u8; 192], PrecompileError> {
        crate::bls12_381::crypto_backend::p2_msm_bytes(pairs)
    }

    /// BLS12-381 pairing check.
    fn bls12_381_pairing_check(
        &self,
        pairs: &[(G1Point, G2Point)],
    ) -> Result<bool, PrecompileError> {
        crate::bls12_381::crypto_backend::pairing_check_bytes(pairs)
    }

    /// BLS12-381 map field element to G1.
    fn bls12_381_fp_to_g1(&self, fp: &[u8; 48]) -> Result<[u8; 96], PrecompileError> {
        crate::bls12_381::crypto_backend::map_fp_to_g1_bytes(fp)
    }

    /// BLS12-381 map field element to G2.
    fn bls12_381_fp2_to_g2(&self, fp2: ([u8; 48], [u8; 48])) -> Result<[u8; 192], PrecompileError> {
        crate::bls12_381::crypto_backend::map_fp2_to_g2_bytes(&fp2.0, &fp2.1)
    }
}

/// Precompile function type. Takes input, gas limit, and crypto implementation and returns precompile result.
pub type PrecompileFn = fn(&[u8], u64) -> PrecompileResult;

/// Precompile error type.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
    /// Bn254 errors
    Bn254FieldPointNotAMember,
    /// Bn254 affine g failed to create
    Bn254AffineGFailedToCreate,
    /// Bn254 pair length
    Bn254PairLength,
    // Blob errors
    /// The input length is not exactly 192 bytes
    BlobInvalidInputLength,
    /// The commitment does not match the versioned hash
    BlobMismatchedVersion,
    /// The proof verification failed
    BlobVerifyKzgProofFailed,
    /// Non-canonical field element
    NonCanonicalFp,
    /// BLS12-381 G1 point not on curve
    Bls12381G1NotOnCurve,
    /// BLS12-381 G1 point not in correct subgroup
    Bls12381G1NotInSubgroup,
    /// BLS12-381 G2 point not on curve
    Bls12381G2NotOnCurve,
    /// BLS12-381 G2 point not in correct subgroup
    Bls12381G2NotInSubgroup,
    /// BLS12-381 scalar input length error
    Bls12381ScalarInputLength,
    /// BLS12-381 G1 add input length error
    Bls12381G1AddInputLength,
    /// BLS12-381 G1 msm input length error
    Bls12381G1MsmInputLength,
    /// BLS12-381 G2 add input length error
    Bls12381G2AddInputLength,
    /// BLS12-381 G2 msm input length error
    Bls12381G2MsmInputLength,
    /// BLS12-381 pairing input length error
    Bls12381PairingInputLength,
    /// BLS12-381 map fp to g1 input length error
    Bls12381MapFpToG1InputLength,
    /// BLS12-381 map fp2 to g2 input length error
    Bls12381MapFp2ToG2InputLength,
    /// BLS12-381 padding error
    Bls12381FpPaddingInvalid,
    /// BLS12-381 fp padding length error
    Bls12381FpPaddingLength,
    /// BLS12-381 g1 padding length error
    Bls12381G1PaddingLength,
    /// BLS12-381 g2 padding length error
    Bls12381G2PaddingLength,
    /// KZG invalid G1 point
    KzgInvalidG1Point,
    /// KZG G1 point not on curve
    KzgG1PointNotOnCurve,
    /// KZG G1 point not in correct subgroup
    KzgG1PointNotInSubgroup,
    /// KZG input length error
    KzgInvalidInputLength,
    /// secp256k1 ecrecover failed
    Secp256k1RecoverFailed,
    /// Isthmus G1 msm input length error
    IsthmusG1MsmInputLength,
    /// Isthmus G2 msm input length error
    IsthmusG2MsmInputLength,
    /// Isthmus pairing input length error
    IsthmusPairingInputLength,
    /// Custom precompile input length error
    InputLength,
    /// Custom precompile storage operation failed
    StorageOperationFailed,
    /// Custom precompile balance operation failed
    BalanceOperationFailed,
    /// Custom precompile transfer operation failed
    TransferFailed,
}

impl PrecompileError {
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
            Self::Bn254FieldPointNotAMember => "field point not a member of bn254 curve",
            Self::Bn254AffineGFailedToCreate => "failed to create affine g point for bn254 curve",
            Self::Bn254PairLength => "bn254 invalid pair length",
            Self::BlobInvalidInputLength => "invalid blob input length",
            Self::BlobMismatchedVersion => "mismatched blob version",
            Self::BlobVerifyKzgProofFailed => "verifying blob kzg proof failed",
            Self::NonCanonicalFp => "non-canonical field element",
            Self::Bls12381G1NotOnCurve => "bls12-381 g1 point not on curve",
            Self::Bls12381G1NotInSubgroup => "bls12-381 g1 point not in correct subgroup",
            Self::Bls12381G2NotOnCurve => "bls12-381 g2 point not on curve",
            Self::Bls12381G2NotInSubgroup => "bls12-381 g2 point not in correct subgroup",
            Self::Bls12381ScalarInputLength => "bls12-381 scalar input length error",
            Self::Bls12381G1AddInputLength => "bls12-381 g1 add input length error",
            Self::Bls12381G1MsmInputLength => "bls12-381 g1 msm input length error",
            Self::Bls12381G2AddInputLength => "bls12-381 g2 add input length error",
            Self::Bls12381G2MsmInputLength => "bls12-381 g2 msm input length error",
            Self::Bls12381PairingInputLength => "bls12-381 pairing input length error",
            Self::Bls12381MapFpToG1InputLength => "bls12-381 map fp to g1 input length error",
            Self::Bls12381MapFp2ToG2InputLength => "bls12-381 map fp2 to g2 input length error",
            Self::Bls12381FpPaddingInvalid => "bls12-381 fp 64 top bytes of input are not zero",
            Self::Bls12381FpPaddingLength => "bls12-381 fp padding length error",
            Self::Bls12381G1PaddingLength => "bls12-381 g1 padding length error",
            Self::Bls12381G2PaddingLength => "bls12-381 g2 padding length error",
            Self::KzgInvalidG1Point => "kzg invalid g1 point",
            Self::KzgG1PointNotOnCurve => "kzg g1 point not on curve",
            Self::KzgG1PointNotInSubgroup => "kzg g1 point not in correct subgroup",
            Self::KzgInvalidInputLength => "kzg invalid input length",
            Self::Secp256k1RecoverFailed => "secp256k1 signature recovery failed",
            Self::IsthmusG1MsmInputLength => {
                "g1 msm input length too long for OP Stack input size limitation"
            }
            Self::IsthmusG2MsmInputLength => {
                "g2 msm input length too long for OP Stack input size limitation"
            }
            Self::IsthmusPairingInputLength => {
                "pairing input length too long for OP Stack input size limitation"
            }
            Self::InputLength => "invalid input length",
            Self::StorageOperationFailed => "storage operation failed",
            Self::BalanceOperationFailed => "balance operation failed",
            Self::TransferFailed => "transfer operation failed",
        };
        f.write_str(s)
    }
}

/// Default implementation of the Crypto trait using the existing crypto libraries.
#[derive(Clone, Debug)]
pub struct DefaultCrypto;

impl Crypto for DefaultCrypto {}
