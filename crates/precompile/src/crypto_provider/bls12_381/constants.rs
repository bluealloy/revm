//! BLS12-381 cryptographic constants
//!
//! These constants define the sizes of various BLS12-381 cryptographic primitives.

/// FP_LENGTH specifies the number of bytes needed to represent an
/// Fp element. This is an element in the base field of BLS12-381.
///
/// Note: The base field is used to define G1 and G2 elements.
pub const FP_LENGTH: usize = 48;

/// SCALAR_LENGTH specifies the number of bytes needed to represent an Fr element.
/// This is an element in the scalar field of BLS12-381.
///
/// Note: Since it is already 32 byte aligned, there is no padded version of this constant.
pub const SCALAR_LENGTH: usize = 32;

/// G1_LENGTH specifies the number of bytes needed to represent a G1 element.
///
/// Note: A G1 element contains 2 Fp elements.
pub const G1_LENGTH: usize = 2 * FP_LENGTH;

/// FP2_LENGTH specifies the number of bytes needed to represent a Fp^2 element.
///
/// Note: This is the quadratic extension of Fp, and by definition
/// means we need 2 Fp elements.
pub const FP2_LENGTH: usize = 2 * FP_LENGTH;

/// G2_LENGTH specifies the number of bytes needed to represent a G2 element.
///
/// Note: A G2 element contains 2 Fp^2 elements.
pub const G2_LENGTH: usize = 2 * FP2_LENGTH;

/// SCALAR_LENGTH_BITS specifies the number of bits needed to represent an Fr element.
/// This is an element in the scalar field of BLS12-381.
pub const SCALAR_LENGTH_BITS: usize = SCALAR_LENGTH * 8;
