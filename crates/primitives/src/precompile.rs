use alloc::vec::Vec;

/// A precompile operation result.
pub type PrecompileResult = Result<(u64, Vec<u8>), PrecompileError>;

pub type StandardPrecompileFn = fn(&[u8], u64) -> PrecompileResult;
pub type CustomPrecompileFn = fn(&[u8], u64) -> PrecompileResult;

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
}
