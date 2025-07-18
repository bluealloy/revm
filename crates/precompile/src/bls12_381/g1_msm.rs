//! BLS12-381 G1 msm precompile. More details in [`g1_msm`]
use super::crypto_backend::p1_msm_bytes;
use super::G1Point;
use crate::bls12_381::utils::{pad_g1_point, remove_g1_padding};
use crate::bls12_381_const::{
    DISCOUNT_TABLE_G1_MSM, G1_MSM_ADDRESS, G1_MSM_BASE_GAS_FEE, G1_MSM_INPUT_LENGTH,
    PADDED_G1_LENGTH, SCALAR_LENGTH,
};
use crate::bls12_381_utils::msm_required_gas;
use crate::{PrecompileError, PrecompileOutput, PrecompileResult, PrecompileWithAddress};
use std::string::ToString;
use std::vec::Vec;

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

    let mut point_scalar_pairs: Vec<(G1Point, [u8; SCALAR_LENGTH])> = Vec::with_capacity(k);

    for i in 0..k {
        let encoded_g1_element =
            &input[i * G1_MSM_INPUT_LENGTH..i * G1_MSM_INPUT_LENGTH + PADDED_G1_LENGTH];
        let encoded_scalar = &input[i * G1_MSM_INPUT_LENGTH + PADDED_G1_LENGTH
            ..i * G1_MSM_INPUT_LENGTH + PADDED_G1_LENGTH + SCALAR_LENGTH];

        let [a_x, a_y] = remove_g1_padding(encoded_g1_element)?;

        // Convert scalar to fixed-size array
        let scalar_array: [u8; SCALAR_LENGTH] = encoded_scalar
            .try_into()
            .map_err(|_| PrecompileError::Other("Invalid scalar length".to_string()))?;

        // Note: We include zero scalars to ensure point validation happens
        // The actual filtering will be done inside p1_msm_bytes if needed
        point_scalar_pairs.push(((*a_x, *a_y), scalar_array));
    }

    // Get unpadded result from crypto backend
    let unpadded_result = p1_msm_bytes(&point_scalar_pairs)?;

    // Pad the result for EVM compatibility
    let padded_result = pad_g1_point(&unpadded_result);

    Ok(PrecompileOutput::new(required_gas, padded_result.into()))
}

#[cfg(test)]
mod test {
    use super::*;
    use primitives::{hex, Bytes};

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
