use super::{
    g2::check_canonical_fp2,
    g2::encode_g2_point,
    utils::{remove_padding, PADDED_FP2_LENGTH, PADDED_FP_LENGTH},
};
use crate::{u64_to_address, PrecompileWithAddress};
use crate::{Precompile, PrecompileError, PrecompileOutput, PrecompileResult};
use blst::{blst_map_to_g2, blst_p2, blst_p2_affine, blst_p2_to_affine};
use primitives::Bytes;

/// [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537#specification) BLS12_MAP_FP2_TO_G2 precompile.
pub const PRECOMPILE: PrecompileWithAddress =
    PrecompileWithAddress(u64_to_address(ADDRESS), Precompile::Standard(map_fp2_to_g2));

/// BLS12_MAP_FP2_TO_G2 precompile address.
pub const ADDRESS: u64 = 0x13;

/// Base gas fee for BLS12-381 map_fp2_to_g2 operation.
const BASE_GAS_FEE: u64 = 75000;

/// Field-to-curve call expects 128 bytes as an input that is interpreted as
/// an element of Fp2. Output of this call is 256 bytes and is an encoded G2
/// point.
/// See also: <https://eips.ethereum.org/EIPS/eip-2537#abi-for-mapping-fp2-element-to-g2-point>
pub(super) fn map_fp2_to_g2(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    if BASE_GAS_FEE > gas_limit {
        return Err(PrecompileError::OutOfGas.into());
    }

    if input.len() != PADDED_FP2_LENGTH {
        return Err(PrecompileError::Other(format!(
            "MAP_FP2_TO_G2 input should be {PADDED_FP2_LENGTH} bytes, was {}",
            input.len()
        ))
        .into());
    }

    let input_p0_x = remove_padding(&input[..PADDED_FP_LENGTH])?;
    let input_p0_y = remove_padding(&input[PADDED_FP_LENGTH..PADDED_FP2_LENGTH])?;
    let fp2 = check_canonical_fp2(input_p0_x, input_p0_y)?;

    let mut p = blst_p2::default();
    // SAFETY: p and fp2 are blst values.
    // third argument is unused if null.
    unsafe { blst_map_to_g2(&mut p, &fp2, core::ptr::null()) };

    let mut p_aff = blst_p2_affine::default();
    // SAFETY: p_aff and p are blst values.
    unsafe { blst_p2_to_affine(&mut p_aff, &p) };

    let out = encode_g2_point(&p_aff);
    Ok(PrecompileOutput::new(BASE_GAS_FEE, out))
}
