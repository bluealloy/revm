//! BLS12-381 map fp2 to g2 precompile. More details in [`map_fp2_to_g2`]
use super::{
    crypto_backend::{encode_g2_point, map_fp2_to_g2 as blst_map_fp2_to_g2, read_fp2},
    utils::remove_fp_padding,
};
use crate::bls12_381_const::{
    MAP_FP2_TO_G2_ADDRESS, MAP_FP2_TO_G2_BASE_GAS_FEE, PADDED_FP2_LENGTH, PADDED_FP_LENGTH,
};
use crate::PrecompileWithAddress;
use crate::{PrecompileError, PrecompileOutput, PrecompileResult};

/// [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537#specification) BLS12_MAP_FP2_TO_G2 precompile.
pub const PRECOMPILE: PrecompileWithAddress =
    PrecompileWithAddress(MAP_FP2_TO_G2_ADDRESS, map_fp2_to_g2);

/// Field-to-curve call expects 128 bytes as an input that is interpreted as
/// an element of Fp2. Output of this call is 256 bytes and is an encoded G2
/// point.
/// See also: <https://eips.ethereum.org/EIPS/eip-2537#abi-for-mapping-fp2-element-to-g2-point>
pub fn map_fp2_to_g2(input: &[u8], gas_limit: u64) -> PrecompileResult {
    if MAP_FP2_TO_G2_BASE_GAS_FEE > gas_limit {
        return Err(PrecompileError::OutOfGas);
    }

    if input.len() != PADDED_FP2_LENGTH {
        return Err(PrecompileError::Other(format!(
            "MAP_FP2_TO_G2 input should be {PADDED_FP2_LENGTH} bytes, was {}",
            input.len()
        )));
    }

    let input_p0_x = remove_fp_padding(&input[..PADDED_FP_LENGTH])?;
    let input_p0_y = remove_fp_padding(&input[PADDED_FP_LENGTH..PADDED_FP2_LENGTH])?;
    let fp2 = read_fp2(input_p0_x, input_p0_y)?;
    let p_aff = blst_map_fp2_to_g2(&fp2);

    let out = encode_g2_point(&p_aff);
    Ok(PrecompileOutput::new(
        MAP_FP2_TO_G2_BASE_GAS_FEE,
        out.into(),
    ))
}
