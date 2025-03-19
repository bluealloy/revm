use super::blst::p2_add_affine;
use super::g2::{encode_g2_point, extract_g2_input_no_subgroup_check};
use crate::bls12_381_const::{
    G2_ADD_ADDRESS, G2_ADD_BASE_GAS_FEE, G2_ADD_INPUT_LENGTH, PADDED_G2_LENGTH,
};
use crate::PrecompileWithAddress;
use crate::{PrecompileError, PrecompileOutput, PrecompileResult};
use primitives::Bytes;

/// [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537#specification) BLS12_G2ADD precompile.
pub const PRECOMPILE: PrecompileWithAddress = PrecompileWithAddress(G2_ADD_ADDRESS, g2_add);

/// G2 addition call expects `512` bytes as an input that is interpreted as byte
/// concatenation of two G2 points (`256` bytes each).
///
/// Output is an encoding of addition operation result - single G2 point (`256`
/// bytes).
/// See also <https://eips.ethereum.org/EIPS/eip-2537#abi-for-g2-addition>
pub(super) fn g2_add(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    if G2_ADD_BASE_GAS_FEE > gas_limit {
        return Err(PrecompileError::OutOfGas);
    }

    if input.len() != G2_ADD_INPUT_LENGTH {
        return Err(PrecompileError::Other(format!(
            "G2ADD input should be {G2_ADD_INPUT_LENGTH} bytes, was {}",
            input.len()
        )));
    }

    // NB: There is no subgroup check for the G2 addition precompile because the time to do the subgroup
    // check would be more than the time it takes to to do the g1 addition.
    //
    // Users should be careful to note whether the points being added are indeed in the right subgroup.
    let a_aff = &extract_g2_input_no_subgroup_check(&input[..PADDED_G2_LENGTH])?;
    let b_aff = &extract_g2_input_no_subgroup_check(&input[PADDED_G2_LENGTH..])?;

    // Use the safe wrapper for G2 point addition
    let p_aff = p2_add_affine(a_aff, b_aff);

    let out = encode_g2_point(&p_aff);
    Ok(PrecompileOutput::new(G2_ADD_BASE_GAS_FEE, out))
}
