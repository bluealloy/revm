use core::fmt;
use primitives::Bytes;
use std::string::String;
use context_interface::result::EVMError;

/// A precompile operation result.
///
/// Returns either `Ok((gas_used, return_bytes))` or `Err(error)`.
pub type PrecompileResult = Result<PrecompileOutput, PrecompileErrors>;

/// Precompile execution output
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PrecompileOutput {
    /// Gas used by the precompile.
    pub gas_used: u64,
    /// Output bytes.
    pub bytes: Bytes,
}

impl PrecompileOutput {
    /// Returns new precompile output with the given gas used and output bytes.
    pub fn new(gas_used: u64, bytes: Bytes) -> Self {
        Self { gas_used, bytes }
    }
}

pub type PrecompileFn = fn(&Bytes, u64) -> PrecompileResult;

/// Precompile errors.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PrecompileErrors {
    Error(PrecompileError),
    Fatal { msg: String },
}

impl<DB, TXERROR> From<PrecompileErrors> for EVMError<DB, TXERROR> {
    fn from(value: PrecompileErrors) -> Self {
        Self::Precompile(value.to_string())
    }
}

impl core::error::Error for PrecompileErrors {}

impl fmt::Display for PrecompileErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Error(e) => e.fmt(f),
            Self::Fatal { msg } => f.write_str(msg),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum PrecompileError {
    /// out of gas is the main error. Others are here just for completeness
    OutOfGas,
    // Blake2 errors
    Blake2WrongLength,
    Blake2WrongFinalIndicatorFlag,
    // Modexp errors
    ModexpExpOverflow,
    ModexpBaseOverflow,
    ModexpModOverflow,
    // Bn128 errors
    Bn128FieldPointNotAMember,
    Bn128AffineGFailedToCreate,
    Bn128PairLength,
    // Blob errors
    /// The input length is not exactly 192 bytes.
    BlobInvalidInputLength,
    /// The commitment does not match the versioned hash.
    BlobMismatchedVersion,
    /// The proof verification failed.
    BlobVerifyKzgProofFailed,
    /// Catch-all variant for other errors.
    Other(String),
}

impl PrecompileError {
    /// Returns an other error with the given message.
    pub fn other(err: impl Into<String>) -> Self {
        Self::Other(err.into())
    }

    /// Returns true if the error is out of gas.
    pub fn is_oog(&self) -> bool {
        matches!(self, Self::OutOfGas)
    }
}

impl From<PrecompileError> for PrecompileErrors {
    fn from(err: PrecompileError) -> Self {
        PrecompileErrors::Error(err)
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
            Self::Bn128FieldPointNotAMember => "field point not a member of bn128 curve",
            Self::Bn128AffineGFailedToCreate => "failed to create affine g point for bn128 curve",
            Self::Bn128PairLength => "bn128 invalid pair length",
            Self::BlobInvalidInputLength => "invalid blob input length",
            Self::BlobMismatchedVersion => "mismatched blob version",
            Self::BlobVerifyKzgProofFailed => "verifying blob kzg proof failed",
            Self::Other(s) => s,
        };
        f.write_str(s)
    }
}
