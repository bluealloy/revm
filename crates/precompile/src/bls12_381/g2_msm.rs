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

    let input_chunks = input.chunks_exact(G2_MSM_INPUT_LENGTH);
    let k = input_chunks.len();
    let required_gas = msm_required_gas(k, &DISCOUNT_TABLE_G2_MSM, G2_MSM_BASE_GAS_FEE);
    if required_gas > gas_limit {
        return Err(PrecompileHalt::OutOfGas);
    }

    let mut valid_pairs_iter = input_chunks.map(|pair| {
        let (padded_g2, scalar_bytes) = pair.split_at(PADDED_G2_LENGTH);

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
