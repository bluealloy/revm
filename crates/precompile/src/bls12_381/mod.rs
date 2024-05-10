use crate::PrecompileWithAddress;

mod g1;
pub mod g1_add;
pub mod g1_msm;
pub mod g1_mul;
mod g2;
pub mod g2_add;
pub mod g2_msm;
pub mod g2_mul;
pub mod map_fp2_to_g2;
pub mod map_fp_to_g1;
mod msm;
pub mod pairing;
mod utils;

/// Returns the BLS12-381 precompiles with their addresses.
pub fn precompiles() -> impl Iterator<Item = PrecompileWithAddress> {
    [
        g1_add::PRECOMPILE,
        g1_mul::PRECOMPILE,
        g1_msm::PRECOMPILE,
        g2_add::PRECOMPILE,
        g2_mul::PRECOMPILE,
        g2_msm::PRECOMPILE,
        pairing::PRECOMPILE,
        map_fp_to_g1::PRECOMPILE,
        map_fp2_to_g2::PRECOMPILE,
    ]
    .into_iter()
}
