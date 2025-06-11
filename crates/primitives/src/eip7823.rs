//! EIP-7823 Set upper bounds for MODEXP
//!
//! Introduce an upper bound on the inputs of the MODEXP precompile.
//! This can reduce the number of potential bugs, because the testing surface is not infinite anymore,
//! and makes it easier to be replaced using EVMMAX.

/// Each of the modexp length inputs (length_of_BASE, length_of_EXPONENT and length_of_MODULUS)
/// MUST be less than or equal to 8192 bits (1024 bytes).
pub const INPUT_SIZE_LIMIT: usize = 1024;
