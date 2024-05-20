use super::{
    g2::{encode_g2_point, extract_g2_input, G2_INPUT_ITEM_LENGTH},
    utils::{extract_scalar_input, NBITS},
};
use crate::{u64_to_address, PrecompileWithAddress};
use blst::{blst_p2, blst_p2_affine, blst_p2_from_affine, blst_p2_mult, blst_p2_to_affine};
use revm_primitives::{Bytes, Precompile, PrecompileError, PrecompileResult};

/// [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537#specification) BLS12_G2MUL precompile.
pub const PRECOMPILE: PrecompileWithAddress =
    PrecompileWithAddress(u64_to_address(ADDRESS), Precompile::Standard(g2_mul));
/// BLS12_G2MUL precompile address.
pub const ADDRESS: u64 = 0x0f;
/// Base gas fee for BLS12-381 g2_mul operation.
pub(super) const BASE_GAS_FEE: u64 = 45000;

/// Input length of g2_mul operation.
pub(super) const INPUT_LENGTH: usize = 288;

/// G2 multiplication call expects `288` bytes as an input that is interpreted as
/// byte concatenation of encoding of G2 point (`256` bytes) and encoding of a
/// scalar value (`32` bytes).
/// Output is an encoding of multiplication operation result - single G2 point
/// (`256` bytes).
/// See also: <https://eips.ethereum.org/EIPS/eip-2537#abi-for-g2-multiplication>
fn g2_mul(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    if BASE_GAS_FEE > gas_limit {
        return Err(PrecompileError::OutOfGas);
    }
    if input.len() != INPUT_LENGTH {
        return Err(PrecompileError::Other(format!(
            "G2MUL input should be {INPUT_LENGTH} bytes, was {}",
            input.len()
        )));
    }

    let p0_aff = &extract_g2_input(&input[..G2_INPUT_ITEM_LENGTH])?;
    let mut p0 = blst_p2::default();
    // SAFETY: p0 and p0_aff are blst values.
    unsafe { blst_p2_from_affine(&mut p0, p0_aff) };

    let input_scalar0 = extract_scalar_input(&input[G2_INPUT_ITEM_LENGTH..])?;

    let mut p = blst_p2::default();
    // SAFETY: input_scalar0.b has fixed size, p and p0 are blst values.
    unsafe { blst_p2_mult(&mut p, &p0, input_scalar0.b.as_ptr(), NBITS) };
    let mut p_aff = blst_p2_affine::default();
    // SAFETY: p_aff and p are blst values.
    unsafe { blst_p2_to_affine(&mut p_aff, &p) };

    let out = encode_g2_point(&p_aff);
    Ok((BASE_GAS_FEE, out))
}
