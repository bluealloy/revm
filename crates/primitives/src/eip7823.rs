//! EIP-7823: Set Upper Bounds for MODEXP
//!
//! Introduces an upper bound on the inputs of the MODEXP precompile.
//! This reduces the number of potential bugs and makes it easier to replace using EVMMAX.

/// Each of the modexp length inputs (length_of_BASE, length_of_EXPONENT and length_of_MODULUS)
/// MUST be less than or equal to 8192 bits (1024 bytes).
pub const INPUT_SIZE_LIMIT: usize = 1024;
