//! Constants for BN128 cryptographic operations

/// Field element length
pub const FQ_LEN: usize = 32;

/// Scalar field element length
pub const SCALAR_LEN: usize = 32;

/// Quadratic extension field element length (Fq2)
pub const FQ2_LEN: usize = 2 * FQ_LEN;

/// G1 point length (x, y coordinates)
pub const G1_LEN: usize = 2 * FQ_LEN;
