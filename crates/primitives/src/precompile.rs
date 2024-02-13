use crate::Env;
use alloy_primitives::Bytes;
use core::fmt;

/// A precompile operation result.
///
/// Returns either `Ok((gas_used, return_bytes))` or `Err(error)`.
pub type PrecompileResult = Result<(u64, Bytes), PrecompileError>;

pub type StandardPrecompileFn = fn(&Bytes, u64) -> PrecompileResult;
pub type EnvPrecompileFn = fn(&Bytes, u64, env: &Env) -> PrecompileResult;

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
}

#[cfg(feature = "std")]
impl std::error::Error for PrecompileError {}

impl fmt::Display for PrecompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrecompileError::OutOfGas => write!(f, "out of gas"),
            PrecompileError::Blake2WrongLength => write!(f, "wrong input length for blake2"),
            PrecompileError::Blake2WrongFinalIndicatorFlag => {
                write!(f, "wrong final indicator flag for blake2")
            }
            PrecompileError::ModexpExpOverflow => write!(f, "modexp exp overflow"),
            PrecompileError::ModexpBaseOverflow => write!(f, "modexp base overflow"),
            PrecompileError::ModexpModOverflow => write!(f, "modexp mod overflow"),
            PrecompileError::Bn128FieldPointNotAMember => {
                write!(f, "field point not a member of bn128 curve")
            }
            PrecompileError::Bn128AffineGFailedToCreate => {
                write!(f, "failed to create affine g point for bn128 curve")
            }
            PrecompileError::Bn128PairLength => write!(f, "bn128 invalid pair length"),
            PrecompileError::BlobInvalidInputLength => write!(f, "invalid blob input length"),
            PrecompileError::BlobMismatchedVersion => write!(f, "mismatched blob version"),
            PrecompileError::BlobVerifyKzgProofFailed => {
                write!(f, "verifying blob kzg proof failed")
            }
        }
    }
}
