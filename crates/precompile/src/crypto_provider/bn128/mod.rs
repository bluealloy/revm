//! BN128 cryptographic implementations for the crypto provider

/// BN128 cryptographic constants
pub mod constants;
pub use constants::*;

// Legacy aliases for internal use
pub(crate) use G1_LENGTH as G1_LEN;
pub(crate) use SCALAR_LENGTH as SCALAR_LEN;

cfg_if::cfg_if! {
    if #[cfg(feature = "bn")] {
        /// Substrate backend for BN128 operations
        pub mod substrate;
        pub use substrate::{g1_point_add, g1_point_mul, pairing_check};

        // silence arkworks lint as substrate impl will be used as default if both are enabled.
        use ark_bn254 as _;
        use ark_ff as _;
        use ark_ec as _;
        use ark_serialize as _;
    } else {
        /// Arkworks backend for BN128 operations
        pub mod arkworks;
        pub use arkworks::{g1_point_add, g1_point_mul, pairing_check};
    }
}
