use crate::PrecompileWithAddress;

mod g1;
#[cfg(feature = "blst")]
pub mod g1_add;
#[cfg(feature = "blst")]
pub mod g1_msm;
mod g2;
#[cfg(feature = "blst")]
pub mod g2_add;
#[cfg(feature = "blst")]
pub mod g2_msm;
#[cfg(feature = "blst")]
pub mod map_fp2_to_g2;
#[cfg(feature = "blst")]
pub mod map_fp_to_g1;
#[cfg(feature = "blst")]
pub mod pairing;
mod utils;
pub mod reuse_const;
pub mod msm;

/// Returns the BLS12-381 precompiles with their addresses.
#[cfg(feature = "blst")]
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
