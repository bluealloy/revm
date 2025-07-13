//! BLS12-381 G2 msm precompile. More details in [`g2_msm`]
use super::crypto_backend::{p2_msm_bytes, G2PointScalarPairRef};
use super::utils::remove_g2_padding;
use crate::bls12_381_const::{
    DISCOUNT_TABLE_G2_MSM, G2_MSM_ADDRESS, G2_MSM_BASE_GAS_FEE, G2_MSM_INPUT_LENGTH,
    PADDED_G2_LENGTH, SCALAR_LENGTH,
};
use crate::bls12_381_utils::msm_required_gas;
use crate::{PrecompileError, PrecompileOutput, PrecompileResult, PrecompileWithAddress};
use std::vec::Vec;

/// [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537#specification) BLS12_G2MSM precompile.
pub const PRECOMPILE: PrecompileWithAddress = PrecompileWithAddress(G2_MSM_ADDRESS, g2_msm);

/// Implements EIP-2537 G2MSM precompile.
/// G2 multi-scalar-multiplication call expects `288*k` bytes as an input that is interpreted
/// as byte concatenation of `k` slices each of them being a byte concatenation
/// of encoding of G2 point (`256` bytes) and encoding of a scalar value (`32`
/// bytes).
/// Output is an encoding of multi-scalar-multiplication operation result - single G2
/// point (`256` bytes).
/// See also: <https://eips.ethereum.org/EIPS/eip-2537#abi-for-g2-multiexponentiation>
pub fn g2_msm(input: &[u8], gas_limit: u64) -> PrecompileResult {
    let input_len = input.len();
    if input_len == 0 || input_len % G2_MSM_INPUT_LENGTH != 0 {
        return Err(PrecompileError::Other(format!(
            "G2MSM input length should be multiple of {G2_MSM_INPUT_LENGTH}, was {input_len}",
        )));
    }

    let k = input_len / G2_MSM_INPUT_LENGTH;
    let required_gas = msm_required_gas(k, &DISCOUNT_TABLE_G2_MSM, G2_MSM_BASE_GAS_FEE);
    if required_gas > gas_limit {
        return Err(PrecompileError::OutOfGas);
    }

    let mut point_scalar_pairs: Vec<G2PointScalarPairRef<'_>> = Vec::with_capacity(k);
    
    for i in 0..k {
        let encoded_g2_element =
            &input[i * G2_MSM_INPUT_LENGTH..i * G2_MSM_INPUT_LENGTH + PADDED_G2_LENGTH];
        let encoded_scalar = &input[i * G2_MSM_INPUT_LENGTH + PADDED_G2_LENGTH
            ..i * G2_MSM_INPUT_LENGTH + PADDED_G2_LENGTH + SCALAR_LENGTH];

        // Filter out points infinity as an optimization, since it is a no-op.
        if encoded_g2_element.iter().all(|i| *i == 0) {
            continue;
        }

        let [a_x_0, a_x_1, a_y_0, a_y_1] = remove_g2_padding(encoded_g2_element)?;

        // If the scalar is zero, then this is a no-op.
        // Note: We still need to parse the point first to validate it
        if encoded_scalar.iter().all(|i| *i == 0) {
            // Validate the point by trying to parse it
            // This is done by p2_msm_bytes internally
            continue;
        }

        // Convert scalar to fixed-size array
        let scalar_array: &[u8; SCALAR_LENGTH] = encoded_scalar.try_into()
            .map_err(|_| PrecompileError::Other("Invalid scalar length".to_string()))?;
            
        point_scalar_pairs.push(((a_x_0, a_x_1, a_y_0, a_y_1), scalar_array));
    }

    // Use the byte-oriented API
    let out = p2_msm_bytes(&point_scalar_pairs)?;
    Ok(PrecompileOutput::new(required_gas, out.into()))
}
