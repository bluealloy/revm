//! BN128 cryptographic implementations for the crypto provider

/// FQ_LEN specifies the number of bytes needed to represent an
/// Fq element. This is an element in the base field of BN254.
///
/// Note: The base field is used to define G1 and G2 elements.
pub(super) const FQ_LEN: usize = 32;

/// SCALAR_LEN specifies the number of bytes needed to represent an Fr element.
/// This is an element in the scalar field of BN254.
pub(super) const SCALAR_LEN: usize = 32;

/// FQ2_LEN specifies the number of bytes needed to represent an
/// Fq^2 element.
///
/// Note: This is the quadratic extension of Fq, and by definition
/// means we need 2 Fq elements.
pub(super) const FQ2_LEN: usize = 2 * FQ_LEN;

/// G1_LEN specifies the number of bytes needed to represent a G1 element.
///
/// Note: A G1 element contains 2 Fq elements.
pub(super) const G1_LEN: usize = 2 * FQ_LEN;

/// G2_LEN specifies the number of bytes needed to represent a G2 element.
///
/// Note: A G2 element contains 2 Fq^2 elements.
pub(super) const G2_LEN: usize = 2 * FQ2_LEN;

cfg_if::cfg_if! {
    if #[cfg(feature = "bn")] {
        /// Substrate backend for BN128 operations
        pub mod substrate;
        pub use substrate::{g1_point_add, g1_point_mul, pairing_check};
    } else {
        /// Arkworks backend for BN128 operations
        pub mod arkworks;
        pub use arkworks::{g1_point_add, g1_point_mul, pairing_check};
    }
}