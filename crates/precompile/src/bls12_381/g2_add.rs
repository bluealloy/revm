//! BLS12-381 G2 add precompile. More details in [`g2_add`]
use super::crypto_backend::{encode_g2_point, p2_add_affine, read_g2_no_subgroup_check};
use super::utils::remove_g2_padding;
use crate::bls12_381_const::{
    G2_ADD_ADDRESS, G2_ADD_BASE_GAS_FEE, G2_ADD_INPUT_LENGTH, PADDED_G2_LENGTH,
};
use crate::{PrecompileError, PrecompileOutput, PrecompileResult, PrecompileWithAddress};

/// [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537#specification) BLS12_G2ADD precompile.
pub const PRECOMPILE: PrecompileWithAddress = PrecompileWithAddress(G2_ADD_ADDRESS, g2_add);

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
        return Err(PrecompileError::Other(format!(
            "G2ADD input should be {G2_ADD_INPUT_LENGTH} bytes, was {}",
            input.len()
        )));
    }

    let [a_x_0, a_x_1, a_y_0, a_y_1] = remove_g2_padding(&input[..PADDED_G2_LENGTH])?;
    let [b_x_0, b_x_1, b_y_0, b_y_1] = remove_g2_padding(&input[PADDED_G2_LENGTH..])?;

    // NB: There is no subgroup check for the G2 addition precompile because the time to do the subgroup
    // check would be more than the time it takes to do the g1 addition.
    //
    // Users should be careful to note whether the points being added are indeed in the right subgroup.
    let a_aff = &read_g2_no_subgroup_check(a_x_0, a_x_1, a_y_0, a_y_1)?;
    let b_aff = &read_g2_no_subgroup_check(b_x_0, b_x_1, b_y_0, b_y_1)?;

    // Use the safe wrapper for G2 point addition
    let p_aff = p2_add_affine(a_aff, b_aff);

    let out = encode_g2_point(&p_aff);
    Ok(PrecompileOutput::new(G2_ADD_BASE_GAS_FEE, out.into()))
}
