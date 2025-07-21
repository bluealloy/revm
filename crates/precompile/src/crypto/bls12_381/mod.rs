//! BLS12-381 cryptographic implementations

pub mod constants;

// silence arkworks-bls12-381 lint as blst will be used as default if both are enabled.
cfg_if::cfg_if! {
    if #[cfg(feature = "blst")]{
        use ark_bls12_381 as _;
        use ark_ff as _;
        use ark_ec as _;
        use ark_serialize as _;
    }
}

// Re-export type aliases used by implementations
pub use constants::FP_LENGTH;
/// G1 point represented as two field elements (x, y coordinates)
pub type G1Point = ([u8; FP_LENGTH], [u8; FP_LENGTH]);
/// G2 point represented as four field elements (x0, x1, y0, y1 coordinates)
pub type G2Point = (
    [u8; FP_LENGTH],
    [u8; FP_LENGTH],
    [u8; FP_LENGTH],
    [u8; FP_LENGTH],
);
/// Pairing pair consisting of a G1 point and a G2 point
pub type PairingPair = (G1Point, G2Point);
/// G1 point paired with a scalar for multi-scalar multiplication
pub type G1PointScalarPair = (G1Point, [u8; constants::SCALAR_LENGTH]);
/// G2 point paired with a scalar for multi-scalar multiplication
pub type G2PointScalarPair = (G2Point, [u8; constants::SCALAR_LENGTH]);

cfg_if::cfg_if! {
    if #[cfg(feature = "blst")]{
        mod blst;
        pub use blst::{
            p1_add_affine_bytes,
            p2_add_affine_bytes,
            p1_msm_bytes,
            p2_msm_bytes,
            pairing_check_bytes,
            map_fp_to_g1_bytes,
            map_fp2_to_g2_bytes
        };
    } else {
        mod arkworks;
        pub use arkworks::{
            p1_add_affine_bytes,
            p2_add_affine_bytes,
            p1_msm_bytes,
            p2_msm_bytes,
            pairing_check_bytes,
            map_fp_to_g1_bytes,
            map_fp2_to_g2_bytes
        };
    }
}
