use super::{
    g1::{encode_g1_point, extract_g1_input, G1_INPUT_ITEM_LENGTH},
    utils::{extract_scalar_input, NBITS},
};
use crate::{u64_to_address, PrecompileWithAddress};
use blst::{blst_p1, blst_p1_affine, blst_p1_from_affine, blst_p1_mult, blst_p1_to_affine};
use revm_primitives::{Bytes, Precompile, PrecompileError, PrecompileResult};

/// [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537#specification) BLS12_G1MUL precompile.
pub const PRECOMPILE: PrecompileWithAddress =
    PrecompileWithAddress(u64_to_address(ADDRESS), Precompile::Standard(g1_mul));
/// BLS12_G1MUL precompile address.
pub const ADDRESS: u64 = 0x0c;
/// Base gas fee for BLS12-381 g1_mul operation.
pub(super) const BASE_GAS_FEE: u64 = 12000;

/// Input length of g1_mul operation.
pub(super) const INPUT_LENGTH: usize = 160;

/// G1 multiplication call expects `160` bytes as an input that is interpreted as
/// byte concatenation of encoding of G1 point (`128` bytes) and encoding of a
/// scalar value (`32` bytes).
/// Output is an encoding of multiplication operation result - single G1 point
/// (`128` bytes).
/// See also: <https://eips.ethereum.org/EIPS/eip-2537#abi-for-g1-multiplication>
pub(super) fn g1_mul(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    if BASE_GAS_FEE > gas_limit {
        return Err(PrecompileError::OutOfGas);
    }
    if input.len() != INPUT_LENGTH {
        return Err(PrecompileError::Other(format!(
            "G1MUL input should be {INPUT_LENGTH} bytes, was {}",
            input.len()
        )));
    }

    // NB: Scalar multiplications, MSMs and pairings MUST perform a subgroup check.
    //
    // So we set the subgroup_check flag to `true`
    let p0_aff = &extract_g1_input(&input[..G1_INPUT_ITEM_LENGTH], true)?;
    let mut p0 = blst_p1::default();
    // SAFETY: p0 and p0_aff are blst values.
    unsafe { blst_p1_from_affine(&mut p0, p0_aff) };

    let input_scalar0 = extract_scalar_input(&input[G1_INPUT_ITEM_LENGTH..])?;

    let mut p = blst_p1::default();
    // SAFETY: input_scalar0.b has fixed size, p and p0 are blst values.
    unsafe { blst_p1_mult(&mut p, &p0, input_scalar0.b.as_ptr(), NBITS) };
    let mut p_aff = blst_p1_affine::default();
    // SAFETY: p_aff and p are blst values.
    unsafe { blst_p1_to_affine(&mut p_aff, &p) };

    let out = encode_g1_point(&p_aff);
    Ok((BASE_GAS_FEE, out))
}
