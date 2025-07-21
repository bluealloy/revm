//! Constants for BLS12-381 cryptographic operations

/// Length of a field element in bytes
pub const FP_LENGTH: usize = 48;

/// Length of a G1 point (x, y coordinates)
pub const G1_LENGTH: usize = 2 * FP_LENGTH;

/// Length of a Fp2 element
pub const FP2_LENGTH: usize = 2 * FP_LENGTH;

/// Length of a G2 point
pub const G2_LENGTH: usize = 2 * FP2_LENGTH;

/// Length of a scalar field element
pub const SCALAR_LENGTH: usize = 32;

/// Number of bits in a scalar field element
pub const SCALAR_LENGTH_BITS: usize = SCALAR_LENGTH * 8;