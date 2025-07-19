//! BLS12-381 cryptographic implementations for the crypto provider

use crate::bls12_381_const::FP_LENGTH;

/// G1 point representation as a tuple of (x, y) coordinates, each 48 bytes
pub type G1Point = ([u8; FP_LENGTH], [u8; FP_LENGTH]);

/// G2 point representation as a tuple of (x0, x1, y0, y1) coordinates, each 48 bytes
pub type G2Point = (
    [u8; FP_LENGTH],
    [u8; FP_LENGTH],
    [u8; FP_LENGTH],
    [u8; FP_LENGTH],
);

/// Pairing pair consisting of a G1 point and a G2 point
pub type PairingPair = (G1Point, G2Point);

cfg_if::cfg_if! {
    if #[cfg(feature = "blst")] {
        /// BLST backend for BLS12-381 operations
        pub mod blst;
        pub use blst::{
            p1_add_affine_bytes, p2_add_affine_bytes,
            p1_msm_bytes, p2_msm_bytes,
            pairing_check_bytes,
            map_fp_to_g1_bytes, map_fp2_to_g2_bytes
        };

        // silence arkworks-bls12-381 lint as blst will be used as default if both are enabled.
        use ark_bls12_381 as _;
        use ark_ff as _;
        use ark_ec as _;
        use ark_serialize as _;
    } else {
        /// Arkworks backend for BLS12-381 operations
        pub mod arkworks;
        pub use arkworks::{
            p1_add_affine_bytes, p2_add_affine_bytes,
            p1_msm_bytes, p2_msm_bytes,
            pairing_check_bytes,
            map_fp_to_g1_bytes, map_fp2_to_g2_bytes
        };
    }
}
