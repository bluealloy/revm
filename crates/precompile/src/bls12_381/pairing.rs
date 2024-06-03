use super::{
    g1::{extract_g1_input, G1_INPUT_ITEM_LENGTH},
    g2::{extract_g2_input, G2_INPUT_ITEM_LENGTH},
};
use crate::{u64_to_address, PrecompileWithAddress};
use blst::{blst_final_exp, blst_fp12, blst_fp12_is_one, blst_fp12_mul, blst_miller_loop};
use revm_primitives::{Bytes, Precompile, PrecompileError, PrecompileResult, B256};

/// [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537#specification) BLS12_PAIRING precompile.
pub const PRECOMPILE: PrecompileWithAddress =
    PrecompileWithAddress(u64_to_address(ADDRESS), Precompile::Standard(pairing));
/// BLS12_PAIRING precompile address.
pub const ADDRESS: u64 = 0x11;

/// Multiplier gas fee for BLS12-381 pairing operation.
const PAIRING_MULTIPLIER_BASE: u64 = 43000;
/// Offset gas fee for BLS12-381 pairing operation.
const PAIRING_OFFSET_BASE: u64 = 65000;
/// Input length of paitring operation.
const INPUT_LENGTH: usize = 384;

/// Pairing call expects 384*k (k being a positive integer) bytes as an inputs
/// that is interpreted as byte concatenation of k slices. Each slice has the
/// following structure:
///    * 128 bytes of G1 point encoding
///    * 256 bytes of G2 point encoding
/// Each point is expected to be in the subgroup of order q.
/// Output is a 32 bytes where first 31 bytes are equal to 0x00 and the last byte
/// is 0x01 if pairing result is equal to the multiplicative identity in a pairing
/// target field and 0x00 otherwise.
/// See also: <https://eips.ethereum.org/EIPS/eip-2537#abi-for-pairing>
pub(super) fn pairing(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    let input_len = input.len();
    if input_len == 0 || input_len % INPUT_LENGTH != 0 {
        return Err(PrecompileError::Other(format!(
            "Pairing input length should be multiple of {INPUT_LENGTH}, was {input_len}"
        )));
    }

    let k = input_len / INPUT_LENGTH;
    let required_gas: u64 = PAIRING_MULTIPLIER_BASE * k as u64 + PAIRING_OFFSET_BASE;
    if required_gas > gas_limit {
        return Err(PrecompileError::OutOfGas);
    }

    // accumulator for the fp12 multiplications of the miller loops.
    let mut acc = blst_fp12::default();
    for i in 0..k {
        // NB: Scalar multiplications, MSMs and pairings MUST perform a subgroup check.
        //
        // So we set the subgroup_check flag to `true`
        let p1_aff = &extract_g1_input(
            &input[i * INPUT_LENGTH..i * INPUT_LENGTH + G1_INPUT_ITEM_LENGTH],
            true,
        )?;

        // NB: Scalar multiplications, MSMs and pairings MUST perform a subgroup check.
        //
        // So we set the subgroup_check flag to `true`
        let p2_aff = &extract_g2_input(
            &input[i * INPUT_LENGTH + G1_INPUT_ITEM_LENGTH
                ..i * INPUT_LENGTH + G1_INPUT_ITEM_LENGTH + G2_INPUT_ITEM_LENGTH],
            true,
        )?;

        if i > 0 {
            // after the first slice (i>0) we use cur_ml to store the current
            // miller loop and accumulate with the previous results using a fp12
            // multiplication.
            let mut cur_ml = blst_fp12::default();
            let mut res = blst_fp12::default();
            // SAFETY: res, acc, cur_ml, p1_aff and p2_aff are blst values.
            unsafe {
                blst_miller_loop(&mut cur_ml, p2_aff, p1_aff);
                blst_fp12_mul(&mut res, &acc, &cur_ml);
            }
            acc = res;
        } else {
            // on the first slice (i==0) there is no previous results and no need
            // to accumulate.
            // SAFETY: acc, p1_aff and p2_aff are blst values.
            unsafe {
                blst_miller_loop(&mut acc, p2_aff, p1_aff);
            }
        }
    }

    // SAFETY: ret and acc are blst values.
    let mut ret = blst_fp12::default();
    unsafe {
        blst_final_exp(&mut ret, &acc);
    }

    let mut result: u8 = 0;
    // SAFETY: ret is a blst value.
    unsafe {
        if blst_fp12_is_one(&ret) {
            result = 1;
        }
    }
    Ok((required_gas, B256::with_last_byte(result).into()))
}
