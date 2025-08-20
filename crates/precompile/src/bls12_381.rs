//! BLS12-381 precompiles added in [`EIP-2537`](https://eips.ethereum.org/EIPS/eip-2537)
//! For more details check modules for each precompile.
use crate::Precompile;

#[allow(dead_code)]
pub(crate) mod arkworks;

cfg_if::cfg_if! {
    if #[cfg(feature = "blst")]{
        pub(crate) mod blst;
        pub(crate) use blst as crypto_backend;
    } else {
        pub(crate) use arkworks as crypto_backend;
    }
}

// Re-export type aliases for use in submodules
use crate::bls12_381_const::{FP_LENGTH, SCALAR_LENGTH};
/// G1 point represented as two field elements (x, y coordinates)
pub type G1Point = ([u8; FP_LENGTH], [u8; FP_LENGTH]);
/// G2 point represented as four field elements (x0, x1, y0, y1 coordinates)
pub type G2Point = (
    [u8; FP_LENGTH],
    [u8; FP_LENGTH],
    [u8; FP_LENGTH],
    [u8; FP_LENGTH],
);
/// G1 point and scalar pair for MSM operations
pub type G1PointScalar = (G1Point, [u8; SCALAR_LENGTH]);
/// G2 point and scalar pair for MSM operations
pub type G2PointScalar = (G2Point, [u8; SCALAR_LENGTH]);
type PairingPair = (G1Point, G2Point);

pub mod g1_add;
pub mod g1_msm;
pub mod g2_add;
pub mod g2_msm;
pub mod map_fp2_to_g2;
pub mod map_fp_to_g1;
pub mod pairing;
mod utils;

/// Returns the BLS12-381 precompiles with their addresses.
pub fn precompiles() -> impl Iterator<Item = Precompile> {
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
