use crate::bls12_381_const::{
    G1_ADD_ADDRESS, G1_MSM_ADDRESS, G2_ADD_ADDRESS, G2_MSM_ADDRESS, MAP_FP2_TO_G2_ADDRESS,
    MAP_FP_TO_G1_ADDRESS, MSM_MULTIPLIER, PAIRING_ADDRESS,
};
use crate::{u64_to_address, PrecompileError, PrecompileWithAddress};
/// Implements the gas schedule for G1/G2 Multiscalar-multiplication assuming 30
/// MGas/second, see also: <https://eips.ethereum.org/EIPS/eip-2537#g1g2-multiexponentiation>
#[inline]
pub fn msm_required_gas(k: usize, discount_table: &[u16], multiplication_cost: u64) -> u64 {
    if k == 0 {
        return 0;
    }

    let index = core::cmp::min(k - 1, discount_table.len() - 1);
    let discount = discount_table[index] as u64;

    (k as u64 * discount * multiplication_cost) / MSM_MULTIPLIER
}

pub fn bls12_381_precompiles_not_supported() -> Vec<PrecompileWithAddress> {
    vec![
        PrecompileWithAddress(u64_to_address(0x0a), |_, _| {
            Err(PrecompileError::Fatal(
                "no_std is not supported for BLS12-381 precompiles".into(),
            ))
        }),
        PrecompileWithAddress(u64_to_address(G1_ADD_ADDRESS), |_, _| {
            Err(PrecompileError::Fatal(
                "no_std is not supported for BLS12-381 precompiles".into(),
            ))
        }),
        PrecompileWithAddress(u64_to_address(G1_MSM_ADDRESS), |_, _| {
            Err(PrecompileError::Fatal(
                "no_std is not supported for BLS12-381 precompiles".into(),
            ))
        }),
        PrecompileWithAddress(u64_to_address(G2_ADD_ADDRESS), |_, _| {
            Err(PrecompileError::Fatal(
                "no_std is not supported for BLS12-381 precompiles".into(),
            ))
        }),
        PrecompileWithAddress(u64_to_address(G2_MSM_ADDRESS), |_, _| {
            Err(PrecompileError::Fatal(
                "no_std is not supported for BLS12-381 precompiles".into(),
            ))
        }),
        PrecompileWithAddress(u64_to_address(PAIRING_ADDRESS), |_, _| {
            Err(PrecompileError::Fatal(
                "no_std is not supported for BLS12-381 precompiles".into(),
            ))
        }),
        PrecompileWithAddress(u64_to_address(MAP_FP_TO_G1_ADDRESS), |_, _| {
            Err(PrecompileError::Fatal(
                "no_std is not supported for BLS12-381 precompiles".into(),
            ))
        }),
        PrecompileWithAddress(u64_to_address(MAP_FP2_TO_G2_ADDRESS), |_, _| {
            Err(PrecompileError::Fatal(
                "no_std is not supported for BLS12-381 precompiles".into(),
            ))
        }),
    ]
}
