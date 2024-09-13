use super::g1::{encode_g1_point, extract_g1_input, G1_INPUT_ITEM_LENGTH};
use crate::{u64_to_address, PrecompileWithAddress};
use blst::{
    blst_p1, blst_p1_add_or_double_affine, blst_p1_affine, blst_p1_from_affine, blst_p1_to_affine,
};
use revm_primitives::{Bytes, Precompile, PrecompileError, PrecompileOutput, PrecompileResult};

/// [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537#specification) BLS12_G1ADD precompile.
pub const PRECOMPILE: PrecompileWithAddress =
    PrecompileWithAddress(u64_to_address(ADDRESS), Precompile::Standard(g1_add));
/// BLS12_G1ADD precompile address.
pub const ADDRESS: u64 = 0x0b;
/// Base gas fee for BLS12-381 g1_add operation.
const BASE_GAS_FEE: u64 = 500;

/// Input length of g1_add operation.
const INPUT_LENGTH: usize = 256;

/// G1 addition call expects `256` bytes as an input that is interpreted as byte
/// concatenation of two G1 points (`128` bytes each).
/// Output is an encoding of addition operation result - single G1 point (`128`
/// bytes).
/// See also: <https://eips.ethereum.org/EIPS/eip-2537#abi-for-g1-addition>
pub(super) fn g1_add(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    if BASE_GAS_FEE > gas_limit {
        return Err(PrecompileError::OutOfGas.into());
    }

    if input.len() != INPUT_LENGTH {
        return Err(PrecompileError::Other(format!(
            "G1ADD input should be {INPUT_LENGTH} bytes, was {}",
            input.len()
        ))
        .into());
    }

    // NB: There is no subgroup check for the G1 addition precompile.
    //
    // So we set the subgroup checks here to `false`
    let a_aff = &extract_g1_input(&input[..G1_INPUT_ITEM_LENGTH], false)?;
    let b_aff = &extract_g1_input(&input[G1_INPUT_ITEM_LENGTH..], false)?;

    let mut b = blst_p1::default();
    // SAFETY: b and b_aff are blst values.
    unsafe { blst_p1_from_affine(&mut b, b_aff) };

    let mut p = blst_p1::default();
    // SAFETY: p, b and a_aff are blst values.
    unsafe { blst_p1_add_or_double_affine(&mut p, &b, a_aff) };

    let mut p_aff = blst_p1_affine::default();
    // SAFETY: p_aff and p are blst values.
    unsafe { blst_p1_to_affine(&mut p_aff, &p) };

    let out = encode_g1_point(&p_aff);
    Ok(PrecompileOutput::new(BASE_GAS_FEE, out))
}
