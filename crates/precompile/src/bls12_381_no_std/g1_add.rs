//! The BLS12-381 g1 addition precompile.

use crate::{
    Precompile, PrecompileError, PrecompileOutput, PrecompileResult, PrecompileWithAddress,
    bls12_381_no_std::utils::{extract_g1_input, encode_g1_point, G1_INPUT_ITEM_LENGTH},
};
use bls12_381::{G1Projective, G1Affine};
use revm_primitives::{address, Address, Bytes};

/// [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537#specification) BLS12_G1ADD precompile.
pub const PRECOMPILE: PrecompileWithAddress =
    PrecompileWithAddress(ADDRESS, Precompile::Standard(g1_add));

/// BLS12_G1ADD precompile address.
pub const ADDRESS: Address = address!("000000000000000000000000000000000000000b");

/// Base gas fee for BLS12-381 g1_add operation.
const BASE_GAS_FEE: u64 = 375;

/// Input length of g1_add operation.
const INPUT_LENGTH: usize = 256;

/// G1 addition call expects `256` bytes as an input that is interpreted as byte
/// concatenation of two G1 points (`128` bytes each).
/// Output is an encoding of addition operation result - single G1 point (`128`
/// bytes).
/// See also: <https://eips.ethereum.org/EIPS/eip-2537#abi-for-g1-addition>
pub fn g1_add(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    if BASE_GAS_FEE > gas_limit {
        return Err(PrecompileError::OutOfGas.into());
    }

    if input.len() != INPUT_LENGTH {
        return Err(PrecompileError::Other(format!(
            "G1ADD input should be {INPUT_LENGTH} bytes, was {}",
            input.len()
        ))
        .into());
    }

    // Extract G1 inputs from the input without subgroup check.
    // G1 Addition precompile does _not_ require subgroup check.
    let a_aff = &extract_g1_input(&input[..G1_INPUT_ITEM_LENGTH])?;
    let b_aff = &extract_g1_input(&input[G1_INPUT_ITEM_LENGTH..])?;

    // Perform the addition.
    use core::ops::Add;
    let b_proj: G1Projective = b_aff.into();
    let out: G1Affine = a_aff.add(&b_proj).into();
    let out = encode_g1_point(out);

    Ok(PrecompileOutput::new(BASE_GAS_FEE, out))
}
