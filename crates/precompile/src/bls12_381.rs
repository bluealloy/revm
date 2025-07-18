//! BLS12-381 precompiles added in [`EIP-2537`](https://eips.ethereum.org/EIPS/eip-2537)
//! For more details check modules for each precompile.
use crate::PrecompileWithAddress;

cfg_if::cfg_if! {
    if #[cfg(feature = "blst")]{
        mod blst;
        use blst as crypto_backend;
    } else {
        mod arkworks;
        use arkworks as crypto_backend;
    }
}

// Re-export type aliases for use in submodules
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

pub mod g1_add;
pub mod g1_msm;
pub mod g2_add;
pub mod g2_msm;
pub mod map_fp2_to_g2;
pub mod map_fp_to_g1;
pub mod pairing;
mod utils;

// Public API for crypto provider
pub use crypto_backend::{
    map_fp2_to_g2_bytes, map_fp_to_g1_bytes, p1_add_affine_bytes, p1_msm_bytes,
    p2_add_affine_bytes, p2_msm_bytes, pairing_check_bytes,
};

/// Returns the BLS12-381 precompiles with their addresses.
pub fn precompiles() -> impl Iterator<Item = PrecompileWithAddress> {
    [
        g1_add::PRECOMPILE,
        g1_msm::PRECOMPILE,
        g2_add::PRECOMPILE,
        g2_msm::PRECOMPILE,
        pairing::PRECOMPILE,
        map_fp_to_g1::PRECOMPILE,
        map_fp2_to_g2::PRECOMPILE,
    ]
    .into_iter()
}
