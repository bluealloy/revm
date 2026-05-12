//! Interface for the precompiles. It contains the precompile result type,
//! the precompile output type, and the precompile error type.
use context_interface::result::AnyError;
use core::fmt::{self, Debug};
use primitives::{Bytes, OnceLock};
use std::{borrow::Cow, boxed::Box, string::String, vec::Vec};

use crate::bls12_381::{G1Point, G1PointScalar, G2Point, G2PointScalar};

/// Global crypto provider instance
static CRYPTO: OnceLock<Box<dyn Crypto>> = OnceLock::new();

/// Install a custom crypto provider globally.
pub fn install_crypto<C: Crypto + 'static>(crypto: C) -> bool {
    CRYPTO.set(Box::new(crypto)).is_ok()
}

/// Get the installed crypto provider, or the default if none is installed.
pub fn crypto() -> &'static dyn Crypto {
    CRYPTO.get_or_init(|| Box::new(DefaultCrypto)).as_ref()
}

/// A precompile operation result type for individual Ethereum precompile functions.
///
/// Returns either `Ok(EthPrecompileOutput)` or `Err(PrecompileHalt)`.
pub type EthPrecompileResult = Result<EthPrecompileOutput, PrecompileHalt>;

/// A precompile operation result type for the precompile provider.
///
/// Returns either `Ok(PrecompileOutput)` or `Err(PrecompileError)`.
/// `PrecompileError` only represents fatal errors that abort EVM execution.
pub type PrecompileResult = Result<PrecompileOutput, PrecompileError>;

/// Simple precompile execution output used by individual Ethereum precompile functions.
///
/// Contains only the gas used and output bytes. For the richer output type
/// with state gas accounting and halt support, see [`PrecompileOutput`].
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EthPrecompileOutput {
    /// Gas used by the precompile.
    pub gas_used: u64,
    /// Output bytes
    pub bytes: Bytes,
}

impl EthPrecompileOutput {
    /// Returns new precompile output with the given gas used and output bytes.
    pub const fn new(gas_used: u64, bytes: Bytes) -> Self {
        Self { gas_used, bytes }
    }
}

/// Status of a precompile execution.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PrecompileStatus {
    /// Precompile executed successfully.
    Success,
    /// Precompile reverted (non-fatal, returns remaining gas).
    Revert,
    /// Precompile halted with a specific reason.
    Halt(PrecompileHalt),
}

impl PrecompileStatus {
    /// Returns `true` if the precompile execution was successful or reverted.
    #[inline]
    pub const fn is_success_or_revert(&self) -> bool {
        matches!(self, PrecompileStatus::Success | PrecompileStatus::Revert)
    }

    /// Returns `true` if the precompile execution was reverted or halted.
    #[inline]
    pub const fn is_revert_or_halt(&self) -> bool {
        matches!(self, PrecompileStatus::Revert | PrecompileStatus::Halt(_))
    }

    /// Returns the halt reason if the precompile halted, `None` otherwise.
    #[inline]
    pub const fn halt_reason(&self) -> Option<&PrecompileHalt> {
        match &self {
            PrecompileStatus::Halt(reason) => Some(reason),
            _ => None,
        }
    }

    /// Returns `true` if the precompile execution was successful.
    #[inline]
    pub const fn is_success(&self) -> bool {
        matches!(self, PrecompileStatus::Success)
    }

    /// Returns `true` if the precompile reverted.
    #[inline]
    pub const fn is_revert(&self) -> bool {
        matches!(self, PrecompileStatus::Revert)
    }

    /// Returns `true` if the precompile halted.
    #[inline]
    pub const fn is_halt(&self) -> bool {
        matches!(self, PrecompileStatus::Halt(_))
    }
}

/// Rich precompile execution output with gas accounting and status support.
///
/// This is the output type used at the precompile provider level. It can express
/// successful execution, reverts, and halts (non-fatal errors like out-of-gas).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PrecompileOutput {
    /// Status of the precompile execution.
    pub status: PrecompileStatus,
    /// Regular gas used by the precompile.
    pub gas_used: u64,
    /// Gas refunded by the precompile.
    pub gas_refunded: i64,
    /// State gas used by the precompile.
    pub state_gas_used: u64,
    /// Reservoir gas for EIP-8037.
    pub reservoir: u64,
    /// Output bytes.
    pub bytes: Bytes,
}

impl PrecompileOutput {
    /// Returns a new precompile output from an Ethereum precompile result.
    pub fn from_eth_result(result: EthPrecompileResult, reservoir: u64) -> Self {
        match result {
            Ok(output) => Self::new(output.gas_used, output.bytes, reservoir),
            Err(halt) => Self::halt(halt, reservoir),
        }
    }
    /// Returns a new successful precompile output.
    pub const fn new(gas_used: u64, bytes: Bytes, reservoir: u64) -> Self {
        Self {
            status: PrecompileStatus::Success,
            gas_used,
            gas_refunded: 0,
            state_gas_used: 0,
            reservoir,
            bytes,
        }
    }

    /// Returns a new halted precompile output with the given halt reason.
    pub const fn halt(reason: PrecompileHalt, reservoir: u64) -> Self {
        Self {
            status: PrecompileStatus::Halt(reason),
            gas_used: 0,
            gas_refunded: 0,
            state_gas_used: 0,
            reservoir,
            bytes: Bytes::new(),
        }
    }

    /// Returns a new reverted precompile output.
    pub const fn revert(gas_used: u64, bytes: Bytes, reservoir: u64) -> Self {
        Self {
            status: PrecompileStatus::Revert,
            gas_used,
            gas_refunded: 0,
            state_gas_used: 0,
            reservoir,
            bytes,
        }
    }

    /// Returns `true` if the precompile execution was successful.
    pub const fn is_success(&self) -> bool {
        matches!(self.status, PrecompileStatus::Success)
    }

    /// Returns `true` if the precompile execution was successful.
    #[deprecated(note = "use `is_success` instead")]
    pub const fn is_ok(&self) -> bool {
        self.is_success()
    }

    /// Returns `true` if the precompile reverted.
    pub const fn is_revert(&self) -> bool {
        matches!(self.status, PrecompileStatus::Revert)
    }

    /// Returns `true` if the precompile halted.
    pub const fn is_halt(&self) -> bool {
        matches!(self.status, PrecompileStatus::Halt(_))
    }

    /// Returns the halt reason if the precompile halted, `None` otherwise.
    #[inline]
    pub const fn halt_reason(&self) -> Option<&PrecompileHalt> {
        self.status.halt_reason()
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
    fn bn254_g1_add(&self, p1: &[u8], p2: &[u8]) -> Result<[u8; 64], PrecompileHalt> {
        crate::bn254::crypto_backend::g1_point_add(p1, p2)
    }

    /// BN254 elliptic curve scalar multiplication.
    #[inline]
    fn bn254_g1_mul(&self, point: &[u8], scalar: &[u8]) -> Result<[u8; 64], PrecompileHalt> {
        crate::bn254::crypto_backend::g1_point_mul(point, scalar)
    }

    /// BN254 pairing check.
    #[inline]
    fn bn254_pairing_check(&self, pairs: &[(&[u8], &[u8])]) -> Result<bool, PrecompileHalt> {
        crate::bn254::crypto_backend::pairing_check(pairs)
    }

    /// secp256k1 ECDSA signature recovery.
    #[inline]
    fn secp256k1_ecrecover(
        &self,
        sig: &[u8; 64],
        recid: u8,
        msg: &[u8; 32],
    ) -> Result<[u8; 32], PrecompileHalt> {
        crate::secp256k1::ecrecover_bytes(sig, recid, msg)
            .ok_or(PrecompileHalt::Secp256k1RecoverFailed)
    }

    /// Modular exponentiation.
    #[inline]
    fn modexp(&self, base: &[u8], exp: &[u8], modulus: &[u8]) -> Result<Vec<u8>, PrecompileHalt> {
        Ok(crate::modexp::modexp(base, exp, modulus))
    }

    /// Blake2 compression function.
    #[inline]
    fn blake2_compress(&self, rounds: u32, h: &mut [u64; 8], m: &[u64; 16], t: &[u64; 2], f: bool) {
        crate::blake2::compress(rounds, h, m, t, f);
    }

    /// secp256r1 (P-256) signature verification.
    #[inline]
    fn secp256r1_verify_signature(&self, msg: &[u8; 32], sig: &[u8; 64], pk: &[u8; 64]) -> bool {
        crate::secp256r1::verify_signature(msg, sig, pk).is_some()
    }

    /// KZG point evaluation.
    #[inline]
    fn verify_kzg_proof(
        &self,
        z: &[u8; 32],
        y: &[u8; 32],
        commitment: &[u8; 48],
        proof: &[u8; 48],
    ) -> Result<(), PrecompileHalt> {
        if !crate::kzg_point_evaluation::verify_kzg_proof(commitment, z, y, proof) {
            return Err(PrecompileHalt::BlobVerifyKzgProofFailed);
        }

        Ok(())
    }

    /// BLS12-381 G1 addition (returns 96-byte unpadded G1 point)
    fn bls12_381_g1_add(&self, a: G1Point, b: G1Point) -> Result<[u8; 96], PrecompileHalt> {
        crate::bls12_381::crypto_backend::p1_add_affine_bytes(a, b)
    }

    /// BLS12-381 G1 multi-scalar multiplication (returns 96-byte unpadded G1 point)
    fn bls12_381_g1_msm(
        &self,
        pairs: &mut dyn Iterator<Item = Result<G1PointScalar, PrecompileHalt>>,
    ) -> Result<[u8; 96], PrecompileHalt> {
        crate::bls12_381::crypto_backend::p1_msm_bytes(pairs)
    }

    /// BLS12-381 G2 addition (returns 192-byte unpadded G2 point)
    fn bls12_381_g2_add(&self, a: G2Point, b: G2Point) -> Result<[u8; 192], PrecompileHalt> {
        crate::bls12_381::crypto_backend::p2_add_affine_bytes(a, b)
    }

    /// BLS12-381 G2 multi-scalar multiplication (returns 192-byte unpadded G2 point)
    fn bls12_381_g2_msm(
        &self,
        pairs: &mut dyn Iterator<Item = Result<G2PointScalar, PrecompileHalt>>,
    ) -> Result<[u8; 192], PrecompileHalt> {
        crate::bls12_381::crypto_backend::p2_msm_bytes(pairs)
    }

    /// BLS12-381 pairing check.
    fn bls12_381_pairing_check(
        &self,
        pairs: &[(G1Point, G2Point)],
    ) -> Result<bool, PrecompileHalt> {
        crate::bls12_381::crypto_backend::pairing_check_bytes(pairs)
    }

    /// BLS12-381 map field element to G1.
    fn bls12_381_fp_to_g1(&self, fp: &[u8; 48]) -> Result<[u8; 96], PrecompileHalt> {
        crate::bls12_381::crypto_backend::map_fp_to_g1_bytes(fp)
    }

    /// BLS12-381 map field element to G2.
    fn bls12_381_fp2_to_g2(&self, fp2: ([u8; 48], [u8; 48])) -> Result<[u8; 192], PrecompileHalt> {
        crate::bls12_381::crypto_backend::map_fp2_to_g2_bytes(&fp2.0, &fp2.1)
    }
}

/// Eth precompile function type. Takes input and gas limit, returns an Eth precompile result.
///
/// This is the function signature used by individual Ethereum precompile implementations.
/// Use [`PrecompileFn`] for the higher-level type that returns [`PrecompileOutput`].
pub type PrecompileEthFn = fn(&[u8], u64) -> EthPrecompileResult;

/// Precompile function type. Takes input, gas limit and reservoir, returns a [`PrecompileResult`].
///
/// Returns `Ok(PrecompileOutput)` for successful execution or non-fatal halts,
/// or `Err(PrecompileError)` for fatal/unrecoverable errors that should abort EVM execution.
pub type PrecompileFn = fn(&[u8], u64, u64) -> PrecompileResult;

/// Macro that generates a thin wrapper function converting a [`PrecompileEthFn`] into a [`PrecompileFn`].
///
/// Usage:
/// ```ignore
/// eth_precompile_fn!(my_precompile, my_eth_fn);
/// ```
/// Expands to:
/// ```ignore
/// fn my_precompile(input: &[u8], gas_limit: u64, reservoir: u64) -> PrecompileOutput {
///     call_eth_precompile(my_eth_fn, input, gas_limit, reservoir)
/// }
/// ```
#[macro_export]
macro_rules! eth_precompile_fn {
    ($name:ident, $eth_fn:expr) => {
        fn $name(input: &[u8], gas_limit: u64, reservoir: u64) -> $crate::PrecompileResult {
            Ok($crate::call_eth_precompile(
                $eth_fn, input, gas_limit, reservoir,
            ))
        }
    };
}

/// Calls a [`PrecompileEthFn`] and wraps the result into a [`PrecompileOutput`].
///
/// Use this in wrapper functions to adapt an eth precompile to the [`PrecompileFn`] signature:
/// ```ignore
/// fn my_precompile(input: &[u8], gas_limit: u64, reservoir: u64) -> PrecompileOutput {
///     call_eth_precompile(my_eth_fn, input, gas_limit, reservoir)
/// }
/// ```
#[inline]
pub fn call_eth_precompile(
    f: PrecompileEthFn,
    input: &[u8],
    gas_limit: u64,
    reservoir: u64,
) -> PrecompileOutput {
    match f(input, gas_limit) {
        Ok(output) => PrecompileOutput::new(output.gas_used, output.bytes, reservoir),
        Err(halt) => PrecompileOutput::halt(halt, reservoir),
    }
}

/// Non-fatal halt reasons for precompiles.
///
/// These represent conditions that halt precompile execution but do not abort
/// the entire EVM transaction. They are expressed through [`PrecompileStatus::Halt`]
/// at the provider level.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PrecompileHalt {
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
    /// Catch-all variant for precompile halt reasons without a dedicated variant.
    Other(Cow<'static, str>),
}

impl PrecompileHalt {
    /// Returns another halt reason with the given message.
    pub fn other(err: impl Into<String>) -> Self {
        Self::Other(Cow::Owned(err.into()))
    }

    /// Returns another halt reason with the given static string.
    pub const fn other_static(err: &'static str) -> Self {
        Self::Other(Cow::Borrowed(err))
    }

    /// Returns `true` if the halt reason is out of gas.
    pub const fn is_oog(&self) -> bool {
        matches!(self, Self::OutOfGas)
    }
}

impl core::error::Error for PrecompileHalt {}

impl fmt::Display for PrecompileHalt {
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
            Self::Other(s) => s,
        };
        f.write_str(s)
    }
}

/// Fatal precompile error type.
///
/// These errors represent unrecoverable conditions that abort the entire EVM
/// transaction. They propagate as `EVMError::Custom`.
///
/// For non-fatal halt reasons (like out-of-gas or invalid input), see
/// [`PrecompileHalt`] which is expressed through [`PrecompileStatus::Halt`].
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PrecompileError {
    /// Unrecoverable error that halts EVM execution.
    Fatal(String),
    /// Unrecoverable error that halts EVM execution.
    FatalAny(AnyError),
}

impl PrecompileError {
    /// Returns `true` if the error is `Fatal` or `FatalAny`.
    pub const fn is_fatal(&self) -> bool {
        true
    }
}

impl core::error::Error for PrecompileError {}

impl fmt::Display for PrecompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fatal(s) => write!(f, "fatal: {s}"),
            Self::FatalAny(s) => write!(f, "fatal: {s}"),
        }
    }
}

/// Default implementation of the Crypto trait using the existing crypto libraries.
#[derive(Clone, Debug)]
pub struct DefaultCrypto;

impl Crypto for DefaultCrypto {}
