//! BLS12-381 G1 msm precompile. More details in [`g1_msm`]
use super::utils::remove_g1_padding;
use crate::bls12_381_const::{
    DISCOUNT_TABLE_G1_MSM, G1_MSM_ADDRESS, G1_MSM_BASE_GAS_FEE, G1_MSM_INPUT_LENGTH,
    PADDED_G1_LENGTH, SCALAR_LENGTH,
};
use crate::bls12_381_utils::msm_required_gas;
use crate::{PrecompileError, PrecompileOutput, PrecompileResult, PrecompileWithAddress};
use primitives::Bytes;
use std::vec::Vec;

// Type alias to reduce complexity warnings
type G1PointScalarPair = (([u8; 48], [u8; 48]), [u8; 32]);

/// [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537#specification) BLS12_G1MSM precompile.
pub const PRECOMPILE: PrecompileWithAddress = PrecompileWithAddress(G1_MSM_ADDRESS, g1_msm);

/// Implements EIP-2537 G1MSM precompile.
/// G1 multi-scalar-multiplication call expects `160*k` bytes as an input that is interpreted
/// as byte concatenation of `k` slices each of them being a byte concatenation
/// of encoding of G1 point (`128` bytes) and encoding of a scalar value (`32`
/// bytes).
/// Output is an encoding of multi-scalar-multiplication operation result - single G1
/// point (`128` bytes).
/// See also: <https://eips.ethereum.org/EIPS/eip-2537#abi-for-g1-multiexponentiation>
pub fn g1_msm(input: &[u8], gas_limit: u64) -> PrecompileResult {
    let input_len = input.len();
    if input_len == 0 || input_len % G1_MSM_INPUT_LENGTH != 0 {
        return Err(PrecompileError::Other(format!(
            "G1MSM input length should be multiple of {G1_MSM_INPUT_LENGTH}, was {input_len}",
        )));
    }

    let k = input_len / G1_MSM_INPUT_LENGTH;
    let required_gas = msm_required_gas(k, &DISCOUNT_TABLE_G1_MSM, G1_MSM_BASE_GAS_FEE);
    if required_gas > gas_limit {
        return Err(PrecompileError::OutOfGas);
    }

    let mut point_scalar_pairs: Vec<G1PointScalarPair> = Vec::with_capacity(k);

    for i in 0..k {
        let encoded_g1_element =
            &input[i * G1_MSM_INPUT_LENGTH..i * G1_MSM_INPUT_LENGTH + PADDED_G1_LENGTH];
        let encoded_scalar = &input[i * G1_MSM_INPUT_LENGTH + PADDED_G1_LENGTH
            ..i * G1_MSM_INPUT_LENGTH + PADDED_G1_LENGTH + SCALAR_LENGTH];

        // Filter out points infinity as an optimization, since it is a no-op.
        // Note: Previously, points were being batch converted from Jacobian to Affine.
        // In `blst`, this would essentially, zero out all of the points.
        // Since all points are now in affine, this bug is avoided.
        if encoded_g1_element.iter().all(|i| *i == 0) {
            continue;
        }

        // If the scalar is zero, then this is a no-op.
        if encoded_scalar.iter().all(|i| *i == 0) {
            continue;
        }

        let [a_x, a_y] = remove_g1_padding(encoded_g1_element)?;

        // Convert to fixed-size arrays for the new interface
        let scalar_array: [u8; SCALAR_LENGTH] = encoded_scalar
            .try_into()
            .map_err(|_| PrecompileError::Other("Invalid scalar length".into()))?;

        point_scalar_pairs.push(((*a_x, *a_y), scalar_array));
    }

    // Return the encoding for the point at the infinity according to EIP-2537
    // if there are no points in the MSM.
    const ENCODED_POINT_AT_INFINITY: [u8; PADDED_G1_LENGTH] = [0; PADDED_G1_LENGTH];
    if point_scalar_pairs.is_empty() {
        return Ok(PrecompileOutput::new(
            required_gas,
            Bytes::from_static(&ENCODED_POINT_AT_INFINITY),
        ));
    }

    // Convert to references for the backend interface
    let pair_refs: Vec<_> = point_scalar_pairs
        .iter()
        .map(|((x, y), s)| ((x, y), s))
        .collect();

    #[cfg(target_os = "zkvm")]
    let out = crate::zkvm::bls12_381::p1_msm_bytes(&pair_refs)?;
    #[cfg(not(target_os = "zkvm"))]
    let out = super::crypto_backend::p1_msm_bytes(&pair_refs)?;

    Ok(PrecompileOutput::new(required_gas, out.into()))
}

#[cfg(test)]
mod test {
    use super::*;
    use primitives::hex;

    #[test]
    fn bls_g1multiexp_g1_not_on_curve_but_in_subgroup() {
        let input = Bytes::from(hex!("000000000000000000000000000000000a2833e497b38ee3ca5c62828bf4887a9f940c9e426c7890a759c20f248c23a7210d2432f4c98a514e524b5184a0ddac00000000000000000000000000000000150772d56bf9509469f9ebcd6e47570429fd31b0e262b66d512e245c38ec37255529f2271fd70066473e393a8bead0c30000000000000000000000000000000000000000000000000000000000000000"));
        let fail = g1_msm(&input, G1_MSM_BASE_GAS_FEE);
        assert_eq!(
            fail,
            Err(PrecompileError::Other(
                "Element not on G1 curve".to_string()
            ))
        );
    }
}
