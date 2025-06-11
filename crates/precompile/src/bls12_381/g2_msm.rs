//! BLS12-381 G2 msm precompile. More details in [`g2_msm`]
use super::crypto_backend::{encode_g2_point, p2_msm, read_g2, read_scalar};
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
            "G2MSM input length should be multiple of {}, was {}",
            G2_MSM_INPUT_LENGTH, input_len
        )));
    }

    let k = input_len / G2_MSM_INPUT_LENGTH;
    let required_gas = msm_required_gas(k, &DISCOUNT_TABLE_G2_MSM, G2_MSM_BASE_GAS_FEE);
    if required_gas > gas_limit {
        return Err(PrecompileError::OutOfGas);
    }

    let mut g2_points: Vec<_> = Vec::with_capacity(k);
    let mut scalars = Vec::with_capacity(k);
    for i in 0..k {
        let encoded_g2_element =
            &input[i * G2_MSM_INPUT_LENGTH..i * G2_MSM_INPUT_LENGTH + PADDED_G2_LENGTH];
        let encoded_scalar = &input[i * G2_MSM_INPUT_LENGTH + PADDED_G2_LENGTH
            ..i * G2_MSM_INPUT_LENGTH + PADDED_G2_LENGTH + SCALAR_LENGTH];

        // Filter out points infinity as an optimization, since it is a no-op.
        // Note: Previously, points were being batch converted from Jacobian to Affine. In `blst`, this would essentially,
        // zero out all of the points. Since all points are in affine, this bug is avoided.
        if encoded_g2_element.iter().all(|i| *i == 0) {
            continue;
        }

        let [a_x_0, a_x_1, a_y_0, a_y_1] = remove_g2_padding(encoded_g2_element)?;

        // NB: Scalar multiplications, MSMs and pairings MUST perform a subgroup check.
        //
        // So we set the subgroup_check flag to `true`
        let p0_aff = read_g2(a_x_0, a_x_1, a_y_0, a_y_1)?;

        // If the scalar is zero, then this is a no-op.
        //
        // Note: This check is made after checking that g2 is valid.
        // this is because we want the precompile to error when
        // G2 is invalid, even if the scalar is zero.
        if encoded_scalar.iter().all(|i| *i == 0) {
            continue;
        }

        // Convert affine point to Jacobian coordinates using our helper function
        g2_points.push(p0_aff);
        scalars.push(read_scalar(encoded_scalar)?);
    }

    // Return infinity point if all points are infinity
    if g2_points.is_empty() {
        return Ok(PrecompileOutput::new(
            required_gas,
            [0; PADDED_G2_LENGTH].into(),
        ));
    }

    // Perform multi-scalar multiplication using the safe wrapper
    let multiexp_aff = p2_msm(g2_points, scalars);

    let out = encode_g2_point(&multiexp_aff);
    Ok(PrecompileOutput::new(required_gas, out.into()))
}
