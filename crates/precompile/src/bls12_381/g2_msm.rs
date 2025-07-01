//! BLS12-381 G2 msm precompile. More details in [`g2_msm`]
use super::utils::remove_g2_padding;
use crate::bls12_381_const::{
    DISCOUNT_TABLE_G2_MSM, G2_MSM_ADDRESS, G2_MSM_BASE_GAS_FEE, G2_MSM_INPUT_LENGTH,
    PADDED_G2_LENGTH, SCALAR_LENGTH,
};
use crate::bls12_381_utils::msm_required_gas;
use crate::{PrecompileError, PrecompileOutput, PrecompileResult, PrecompileWithAddress};
use primitives::Bytes;
use std::vec::Vec;

// Type alias to reduce complexity warnings
type G2PointScalarPair = (([u8; 48], [u8; 48], [u8; 48], [u8; 48]), [u8; 32]);

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

    let mut point_scalar_pairs: Vec<G2PointScalarPair> = Vec::with_capacity(k);

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

        // If the scalar is zero, then this is a no-op.
        if encoded_scalar.iter().all(|i| *i == 0) {
            continue;
        }

        let [a_x_0, a_x_1, a_y_0, a_y_1] = remove_g2_padding(encoded_g2_element)?;

        // Convert to fixed-size arrays for the new interface
        let scalar_array: [u8; SCALAR_LENGTH] = encoded_scalar
            .try_into()
            .map_err(|_| PrecompileError::Other("Invalid scalar length".into()))?;

        point_scalar_pairs.push(((*a_x_0, *a_x_1, *a_y_0, *a_y_1), scalar_array));
    }

    // Return the encoding for the point at the infinity according to EIP-2537
    // if there are no points in the MSM.
    const ENCODED_POINT_AT_INFINITY: [u8; PADDED_G2_LENGTH] = [0; PADDED_G2_LENGTH];
    if point_scalar_pairs.is_empty() {
        return Ok(PrecompileOutput::new(
            required_gas,
            Bytes::from_static(&ENCODED_POINT_AT_INFINITY),
        ));
    }

    // Convert to references for the backend interface
    let pair_refs: Vec<_> = point_scalar_pairs
        .iter()
        .map(|((x0, x1, y0, y1), s)| ((x0, x1, y0, y1), s))
        .collect();

    #[cfg(target_os = "zkvm")]
    let out = crate::zkvm::bls12_381::p2_msm_bytes(&pair_refs)?;
    #[cfg(not(target_os = "zkvm"))]
    let out = super::crypto_backend::p2_msm_bytes(&pair_refs)?;

    Ok(PrecompileOutput::new(required_gas, out.into()))
}
