//! Utility functions for the BLS12-381 precompiles
use crate::bls12_381_const::MSM_MULTIPLIER;

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
