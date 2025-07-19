//! BLS12-381 precompiles added in [`EIP-2537`](https://eips.ethereum.org/EIPS/eip-2537)
//! For more details check modules for each precompile.
use crate::PrecompileWithAddress;

// Re-export type aliases from crypto_provider for backward compatibility
pub use crate::crypto_provider::bls12_381::{G1Point, G2Point, PairingPair};

pub mod g1_add;
pub mod g1_msm;
pub mod g2_add;
pub mod g2_msm;
pub mod map_fp2_to_g2;
pub mod map_fp_to_g1;
pub mod pairing;
mod utils;

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
