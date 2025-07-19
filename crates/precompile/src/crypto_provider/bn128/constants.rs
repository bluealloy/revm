//! BN128 (alt_bn128) cryptographic constants
//!
//! These constants define the sizes of various BN128 cryptographic primitives.

/// FQ_LEN specifies the number of bytes needed to represent an
/// Fq element. This is an element in the base field of BN254.
///
/// Note: The base field is used to define G1 and G2 elements.
pub const FQ_LEN: usize = 32;

/// SCALAR_LENGTH specifies the number of bytes needed to represent an Fr element.
/// This is an element in the scalar field of BN254.
pub const SCALAR_LENGTH: usize = 32;

/// FQ2_LEN specifies the number of bytes needed to represent an
/// Fq^2 element.
///
/// Note: This is the quadratic extension of Fq, and by definition
/// means we need 2 Fq elements.
pub const FQ2_LEN: usize = 2 * FQ_LEN;

/// G1_LENGTH specifies the number of bytes needed to represent a G1 element.
///
/// Note: A G1 element contains 2 Fq elements.
pub const G1_LENGTH: usize = 2 * FQ_LEN;

/// G2_LENGTH specifies the number of bytes needed to represent a G2 element.
///
/// Note: A G2 element contains 2 Fq^2 elements.
pub const G2_LENGTH: usize = 2 * FQ2_LEN;
