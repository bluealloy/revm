//! BLS12-381 G1 add precompile. More details in [`g1_add`]
use super::crypto_backend::{encode_g1_point, p1_add_affine, read_g1_no_subgroup_check};
use super::utils::remove_g1_padding;
use crate::bls12_381_const::{
    G1_ADD_ADDRESS, G1_ADD_BASE_GAS_FEE, G1_ADD_INPUT_LENGTH, PADDED_G1_LENGTH,
};
use crate::{PrecompileError, PrecompileOutput, PrecompileResult, PrecompileWithAddress};

/// [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537#specification) BLS12_G1ADD precompile.
pub const PRECOMPILE: PrecompileWithAddress = PrecompileWithAddress(G1_ADD_ADDRESS, g1_add);

/// G1 addition call expects `256` bytes as an input that is interpreted as byte
/// concatenation of two G1 points (`128` bytes each).
/// Output is an encoding of addition operation result - single G1 point (`128`
/// bytes).
/// See also: <https://eips.ethereum.org/EIPS/eip-2537#abi-for-g1-addition>
pub fn g1_add(input: &[u8], gas_limit: u64) -> PrecompileResult {
    if G1_ADD_BASE_GAS_FEE > gas_limit {
        return Err(PrecompileError::OutOfGas);
    }

    if input.len() != G1_ADD_INPUT_LENGTH {
        return Err(PrecompileError::Other(format!(
            "G1ADD input should be {G1_ADD_INPUT_LENGTH} bytes, was {}",
            input.len()
        )));
    }

    let [a_x, a_y] = remove_g1_padding(&input[..PADDED_G1_LENGTH])?;
    let [b_x, b_y] = remove_g1_padding(&input[PADDED_G1_LENGTH..])?;

    // NB: There is no subgroup check for the G1 addition precompile because the time to do the subgroup
    // check would be more than the time it takes to do the g1 addition.
    //
    // Users should be careful to note whether the points being added are indeed in the right subgroup.
    let a_aff = &read_g1_no_subgroup_check(a_x, a_y)?;
    let b_aff = &read_g1_no_subgroup_check(b_x, b_y)?;
    let p_aff = p1_add_affine(a_aff, b_aff);

    let out = encode_g1_point(&p_aff);
    Ok(PrecompileOutput::new(G1_ADD_BASE_GAS_FEE, out.into()))
}
