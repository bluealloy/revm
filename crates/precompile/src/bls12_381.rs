use crate::PrecompileWithAddress;
use cfg_if::cfg_if;

mod g1;
pub mod g1_add;
pub mod g1_msm;
mod g2;
pub mod g2_add;
pub mod g2_msm;
pub mod map_fp2_to_g2;
pub mod map_fp_to_g1;
pub mod pairing;
mod utils;

/// Returns the BLS12-381 precompiles with their addresses.
pub fn precompiles() -> impl Iterator<Item = PrecompileWithAddress> {
    cfg_if! {
        if #[cfg(not(feature = "std"))] {  // If no_std is enabled
            vec![
                PrecompileWithAddress(
                    u64_to_address(0x0A),
                    |_,_| Err(PrecompileError::Fatal("no_std is not supported for BLS12-381 precompiles".into()))
                )
            ].into_iter()
        } else {
            vec![
                g1_add::PRECOMPILE,
                g1_msm::PRECOMPILE,
                g2_add::PRECOMPILE,
                g2_msm::PRECOMPILE,
                pairing::PRECOMPILE,
                map_fp_to_g1::PRECOMPILE,
                map_fp2_to_g2::PRECOMPILE,
            ].into_iter()
        }
    }
}
