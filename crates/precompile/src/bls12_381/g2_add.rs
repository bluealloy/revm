//! BLS12-381 G2 add precompile. More details in [`g2_add`]
use super::utils::{pad_g2_point, remove_g2_padding};
use crate::bls12_381_const::{
    G2_ADD_ADDRESS, G2_ADD_BASE_GAS_FEE, G2_ADD_INPUT_LENGTH, PADDED_G2_LENGTH,
};
use crate::{
    crypto, Precompile, PrecompileError, PrecompileId, PrecompileOutput, PrecompileResult,
};

/// [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537#specification) BLS12_G2ADD precompile.
pub const PRECOMPILE: Precompile =
    Precompile::new(PrecompileId::Bls12G2Add, G2_ADD_ADDRESS, g2_add);

/// G2 addition call expects `512` bytes as an input that is interpreted as byte
/// concatenation of two G2 points (`256` bytes each).
///
/// Output is an encoding of addition operation result - single G2 point (`256`
/// bytes).
/// See also <https://eips.ethereum.org/EIPS/eip-2537#abi-for-g2-addition>
pub fn g2_add(input: &[u8], gas_limit: u64) -> PrecompileResult {
    if G2_ADD_BASE_GAS_FEE > gas_limit {
        return Err(PrecompileError::OutOfGas);
    }

    if input.len() != G2_ADD_INPUT_LENGTH {
        return Err(PrecompileError::Bls12381G2AddInputLength);
    }

    // Extract coordinates from padded input
    let [a_x_0, a_x_1, a_y_0, a_y_1] = remove_g2_padding(&input[..PADDED_G2_LENGTH])?;
    let [b_x_0, b_x_1, b_y_0, b_y_1] = remove_g2_padding(&input[PADDED_G2_LENGTH..])?;

    let a = (*a_x_0, *a_x_1, *a_y_0, *a_y_1);
    let b = (*b_x_0, *b_x_1, *b_y_0, *b_y_1);

    let unpadded_result = crypto().bls12_381_g2_add(a, b)?;

    // Pad the result for EVM compatibility
    let padded_result = pad_g2_point(&unpadded_result);

    Ok(PrecompileOutput::new(
        G2_ADD_BASE_GAS_FEE,
        padded_result.into(),
    ))
}
