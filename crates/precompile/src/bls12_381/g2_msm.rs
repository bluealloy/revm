//! BLS12-381 G2 msm precompile. More details in [`g2_msm`]
use super::utils::{pad_g2_point, remove_g2_padding};
use crate::{
    bls12_381_const::{
        DISCOUNT_TABLE_G2_MSM, G2_MSM_ADDRESS, G2_MSM_BASE_GAS_FEE, G2_MSM_INPUT_LENGTH,
        PADDED_G2_LENGTH, SCALAR_LENGTH,
    },
    bls12_381_utils::msm_required_gas,
    crypto, eth_precompile_fn, EthPrecompileOutput, EthPrecompileResult, Precompile,
    PrecompileHalt, PrecompileId,
};

eth_precompile_fn!(g2_msm_precompile, g2_msm);

/// [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537#specification) BLS12_G2MSM precompile.
pub const PRECOMPILE: Precompile =
    Precompile::new(PrecompileId::Bls12G2Msm, G2_MSM_ADDRESS, g2_msm_precompile);

/// Implements EIP-2537 G2MSM precompile.
/// G2 multi-scalar-multiplication call expects `288*k` bytes as an input that is interpreted
/// as byte concatenation of `k` slices each of them being a byte concatenation
/// of encoding of G2 point (`256` bytes) and encoding of a scalar value (`32`
/// bytes).
/// Output is an encoding of multi-scalar-multiplication operation result - single G2
/// point (`256` bytes).
/// See also: <https://eips.ethereum.org/EIPS/eip-2537#abi-for-g2-multiexponentiation>
pub fn g2_msm(input: &[u8], gas_limit: u64) -> EthPrecompileResult {
    let input_len = input.len();
    if input_len == 0 || !input_len.is_multiple_of(G2_MSM_INPUT_LENGTH) {
        return Err(PrecompileHalt::Bls12381G2MsmInputLength);
    }

    let k = input_len / G2_MSM_INPUT_LENGTH;
    let required_gas = msm_required_gas(k, &DISCOUNT_TABLE_G2_MSM, G2_MSM_BASE_GAS_FEE);
    if required_gas > gas_limit {
        return Err(PrecompileHalt::OutOfGas);
    }

    let mut valid_pairs_iter = (0..k).map(|i| {
        let start = i * G2_MSM_INPUT_LENGTH;
        let padded_g2 = &input[start..start + PADDED_G2_LENGTH];
        let scalar_bytes = &input[start + PADDED_G2_LENGTH..start + G2_MSM_INPUT_LENGTH];

        // Remove padding from G2 point - this validates padding format
        let [x_0, x_1, y_0, y_1] = remove_g2_padding(padded_g2)?;
        let scalar_array: [u8; SCALAR_LENGTH] = scalar_bytes.try_into().unwrap();

        Ok(((*x_0, *x_1, *y_0, *y_1), scalar_array))
    });

    let unpadded_result = crypto().bls12_381_g2_msm(&mut valid_pairs_iter)?;

    // Pad the result for EVM compatibility
    let padded_result = pad_g2_point(&unpadded_result);

    Ok(EthPrecompileOutput::new(required_gas, padded_result.into()))
}

#[cfg(test)]
mod test {
    use super::*;
    use primitives::{hex, Bytes};
    use std::vec::Vec;

    const SCALAR_MODULUS: [u8; SCALAR_LENGTH] =
        hex!("73eda753299d7d483339d80809a1d80553bda402fffe5bfeffffffff00000001");

    fn g2_generator_with_scalar_modulus() -> Bytes {
        let mut input = Vec::with_capacity(G2_MSM_INPUT_LENGTH);
        input.extend_from_slice(&[0u8; 16]);
        input.extend_from_slice(&hex!(
            "024aa2b2f08f0a91260805272dc51051c6e47ad4fa403b02b4510b647ae3d1770bac0326a805bbefd48056c8c121bdb8"
        ));
        input.extend_from_slice(&[0u8; 16]);
        input.extend_from_slice(&hex!(
            "13e02b6052719f607dacd3a088274f65596bd0d09920b61ab5da61bbdc7f5049334cf11213945d57e5ac7d055d042b7e"
        ));
        input.extend_from_slice(&[0u8; 16]);
        input.extend_from_slice(&hex!(
            "0ce5d527727d6e118cc9cdc6da2e351aadfd9baa8cbdd3a76d429a695160d12c923ac9cc3baca289e193548608b82801"
        ));
        input.extend_from_slice(&[0u8; 16]);
        input.extend_from_slice(&hex!(
            "0606c4a02ea734cc32acd2b02bc28b99cb3e287e85a763af267492ab572e99ab3f370d275cec1da1aaa9075ff05f79be"
        ));
        input.extend_from_slice(&SCALAR_MODULUS);
        input.into()
    }

    #[test]
    fn bls_g2msm_scalar_modulus_returns_infinity() {
        let output = g2_msm(&g2_generator_with_scalar_modulus(), G2_MSM_BASE_GAS_FEE).unwrap();
        assert_eq!(output.gas_used, G2_MSM_BASE_GAS_FEE);
        assert_eq!(output.bytes, Bytes::from(vec![0; PADDED_G2_LENGTH]));
    }
}
