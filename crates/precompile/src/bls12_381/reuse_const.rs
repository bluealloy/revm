use crate::PrecompileWithAddress;
use crate::bls12_381::g1_add;
use crate::bls12_381::g1_msm;
use crate::bls12_381::g2_add;
use crate::bls12_381::g2_msm;
use crate::bls12_381::map_fp_to_g1;
use crate::bls12_381::map_fp2_to_g2;
use crate::bls12_381::pairing;

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
