//! Map an element of Fp to a point on G2 curve.

use crate::{
    Precompile, PrecompileError, PrecompileOutput, PrecompileResult, PrecompileWithAddress,
    bls12_381_no_std::utils::{PADDED_FP2_LENGTH, PADDED_FP_LENGTH, encode_g2_point, remove_padding},
};
use bls12_381::G2Affine;
use revm_primitives::{address, Address, Bytes};

/// [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537#specification) BLS12_MAP_FP2_TO_G2 precompile.
pub const PRECOMPILE: PrecompileWithAddress =
    PrecompileWithAddress(ADDRESS, Precompile::Standard(map_fp2_to_g2));

/// BLS12_MAP_FP_TO_G1 precompile address.
pub const ADDRESS: Address = address!("0000000000000000000000000000000000000011");

/// Base gas fee for BLS12-381 map_fp2_to_g2 operation.
const BASE_GAS_FEE: u64 = 23800;

/// Field-to-curve call expects 128 bytes as an input that is interpreted as
/// an element of Fp2. Output of this call is 256 bytes and is an encoded G2
/// point.
/// See also: <https://eips.ethereum.org/EIPS/eip-2537#abi-for-mapping-fp2-element-to-g2-point>
pub fn map_fp2_to_g2(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    if BASE_GAS_FEE > gas_limit {
        return Err(PrecompileError::OutOfGas.into());
    }

    if input.len() != PADDED_FP2_LENGTH {
        return Err(PrecompileError::Other(format!(
            "MAP_FP_TO_G1 input should be {PADDED_FP_LENGTH} bytes, was {}",
            input.len()
        ))
        .into());
    }

    let input_p0_x = remove_padding(&input[..PADDED_FP_LENGTH])?;
    let input_p0_y = remove_padding(&input[PADDED_FP_LENGTH..PADDED_FP2_LENGTH])?;

    // Copy the x and y inputs into the compressed buffer.
    let mut compressed = [0u8; 96];
    compressed[..48].copy_from_slice(&input_p0_x[..]);
    compressed[48..].copy_from_slice(&input_p0_y[..]);
    let aff = G2Affine::from_compressed(&compressed).into_option().ok_or_else(|| {
        PrecompileError::Other("non-canonical fp value".to_string())
    })?;

    let out = encode_g2_point(aff);
    Ok(PrecompileOutput::new(BASE_GAS_FEE, out))
}
