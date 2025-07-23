//! Interface for the precompiles. It contains the precompile result type,
//! the precompile output type, and the precompile error type.
use core::fmt::{self, Debug};
use primitives::Bytes;
use std::{boxed::Box, string::String};

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
}
