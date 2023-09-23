use crate::Env;
use alloc::vec::Vec;

/// A precompile operation result.
///
/// Returns either `Ok((gas_used, return_bytes))` or `Err(error)`.
pub type PrecompileResult = Result<(u64, Vec<u8>), PrecompileError>;

pub type StandardPrecompileFn = fn(&[u8], u64) -> PrecompileResult;
pub type EnvPrecompileFn = fn(&[u8], u64, env: &Env) -> PrecompileResult;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrecompileError {
    /// out of gas is the main error. Other are just here for completeness
    OutOfGas,
    // Blake2 erorr
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
