use super::{
    blst::p1_msm,
    g1::{encode_g1_point, extract_g1_input},
    utils::extract_scalar_input,
};
use crate::bls12_381_const::{
    DISCOUNT_TABLE_G1_MSM, G1_MSM_ADDRESS, G1_MSM_BASE_GAS_FEE, G1_MSM_INPUT_LENGTH, NBITS,
    PADDED_G1_LENGTH, SCALAR_LENGTH,
};
use crate::bls12_381_utils::msm_required_gas;
use crate::PrecompileWithAddress;
use crate::{PrecompileError, PrecompileOutput, PrecompileResult};
use blst::blst_p1_affine;
use primitives::Bytes;

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
pub(super) fn g1_msm(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    let input_len = input.len();
    if input_len == 0 || input_len % G1_MSM_INPUT_LENGTH != 0 {
        return Err(PrecompileError::Other(format!(
            "G1MSM input length should be multiple of {}, was {}",
            G1_MSM_INPUT_LENGTH, input_len
        )));
    }

    let k = input_len / G1_MSM_INPUT_LENGTH;
    let required_gas = msm_required_gas(k, &DISCOUNT_TABLE_G1_MSM, G1_MSM_BASE_GAS_FEE);
    if required_gas > gas_limit {
        return Err(PrecompileError::OutOfGas);
    }

    let mut g1_points: Vec<blst_p1_affine> = Vec::with_capacity(k);
    let mut scalars_bytes: Vec<u8> = Vec::with_capacity(k * SCALAR_LENGTH);
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
        // NB: Scalar multiplications, MSMs and pairings MUST perform a subgroup check.
        let p0_aff = extract_g1_input(encoded_g1_element)?;

        // If the scalar is zero, then this is a no-op.
        //
        // Note: This check is made after checking that g1 is valid.
        // this is because we want the precompile to error when
        // G1 is invalid, even if the scalar is zero.
        if encoded_scalar.iter().all(|i| *i == 0) {
            continue;
        }

        g1_points.push(p0_aff);

        scalars_bytes.extend_from_slice(&extract_scalar_input(encoded_scalar)?.b);
    }

    // Return the encoding for the point at the infinity according to EIP-2537
    // if there are no points in the MSM.
    const ENCODED_POINT_AT_INFINITY: [u8; PADDED_G1_LENGTH] = [0; PADDED_G1_LENGTH];
    if g1_points.is_empty() {
        return Ok(PrecompileOutput::new(
            required_gas,
            ENCODED_POINT_AT_INFINITY.into(),
        ));
    }

    let multiexp_aff = p1_msm(g1_points, scalars_bytes, NBITS);

    let out = encode_g1_point(&multiexp_aff);
    Ok(PrecompileOutput::new(required_gas, out))
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
