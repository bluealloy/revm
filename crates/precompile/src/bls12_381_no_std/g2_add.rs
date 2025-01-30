//! The BLS12-381 g2 addition precompile.

use crate::{
    Precompile, PrecompileError, PrecompileOutput, PrecompileResult, PrecompileWithAddress,
    bls12_381_no_std::utils::{extract_g2_input, encode_g2_point, G2_INPUT_ITEM_LENGTH},
};
use bls12_381::{G2Projective, G2Affine};
use revm_primitives::{address, Address, Bytes};

/// [EIP-2537](https://eips.ethereum.org/EIPS/eip-2537#specification) BLS12_G1ADD precompile.
pub const PRECOMPILE: PrecompileWithAddress =
    PrecompileWithAddress(ADDRESS, Precompile::Standard(g2_add));

/// BLS12_G1ADD precompile address.
pub const ADDRESS: Address = address!("000000000000000000000000000000000000000d");

/// Base gas fee for BLS12-381 g2_add operation.
const BASE_GAS_FEE: u64 = 600;

/// Input length of g2_add operation.
const INPUT_LENGTH: usize = 512;

/// G2 addition call expects `512` bytes as an input that is interpreted as byte
/// concatenation of two G2 points (`256` bytes each).
/// Output is an encoding of addition operation result - single G2 point (`256`
/// bytes).
/// See also: <https://eips.ethereum.org/EIPS/eip-2537#abi-for-g2-addition>
pub fn g2_add(input: &Bytes, gas_limit: u64) -> PrecompileResult {
    if BASE_GAS_FEE > gas_limit {
        return Err(PrecompileError::OutOfGas.into());
    }

    if input.len() != INPUT_LENGTH {
        return Err(PrecompileError::Other(format!(
            "G2ADD input should be {INPUT_LENGTH} bytes, was {}",
            input.len()
        ))
        .into());
    }

    // Extract G2 inputs from the input without subgroup check.
    // G2 Addition precompile does _not_ require subgroup check.
    let a_aff = &extract_g2_input(&input[..G2_INPUT_ITEM_LENGTH])?;
    let b_aff = &extract_g2_input(&input[G2_INPUT_ITEM_LENGTH..])?;

    // Perform the addition.
    use core::ops::Add;
    let b_proj: G2Projective = b_aff.into();
    let out: G2Affine = a_aff.add(&b_proj).into();
    let out = encode_g2_point(out);

    Ok(PrecompileOutput::new(BASE_GAS_FEE, out))
}

