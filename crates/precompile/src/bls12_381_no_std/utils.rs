//! Utilities for working with big endian and fp.

use bls12_381::G1Affine;
use revm_primitives::{Bytes, PrecompileError};

/// Finite field element input length.
pub const FP_LENGTH: usize = 48;

/// Finite field element padded input length.
pub const PADDED_FP_LENGTH: usize = 64;

/// Input elements padding length.
pub const PADDING_LENGTH: usize = 16;

/// Length of each of the elements in a g1 operation input.
pub const G1_INPUT_ITEM_LENGTH: usize = 128;

/// Output length of a g1 operation.
pub const G1_OUTPUT_LENGTH: usize = 128;

/// Encodes a G1 point in affine format into byte slice with padded elements.
pub fn encode_g1_point(input: G1Affine) -> Bytes {
    let uncompressed = input.to_uncompressed();
    let mut out = vec![0u8; G1_OUTPUT_LENGTH];
    out[16..64].copy_from_slice(&uncompressed[..48]);
    out[80..128].copy_from_slice(&uncompressed[48..]);
    out.into()
}

/// Extracts a G1 point in Affine format from a 128 byte slice representation.
///
/// NOTE: This function will perform a G1 subgroup check if `subgroup_check` is set to `true`.
pub fn extract_g1_input(input: &[u8]) -> Result<G1Affine, PrecompileError> {
        if input.len() != G1_INPUT_ITEM_LENGTH {
        return Err(PrecompileError::Other(format!(
            "Input should be {G1_INPUT_ITEM_LENGTH} bytes, was {}",
            input.len()
        )));
    }

    let input_p0_x = remove_padding(&input[..PADDED_FP_LENGTH])?;
    let input_p0_y = remove_padding(&input[PADDED_FP_LENGTH..G1_INPUT_ITEM_LENGTH])?;

    // Fill a new input array with the unpadded values
    let mut new_input: [u8; 96] = [0; 96];
    new_input[..48].copy_from_slice(input_p0_x);
    new_input[48..].copy_from_slice(input_p0_y);

    G1Affine::from_uncompressed(&new_input).into_option().ok_or(PrecompileError::Other("Invalid G1 point".to_string()))
}

/// Removes zeros with which the precompile inputs are left padded to 64 bytes.
pub fn remove_padding(input: &[u8]) -> Result<&[u8; FP_LENGTH], PrecompileError> {
    if input.len() != PADDED_FP_LENGTH {
        return Err(PrecompileError::Other(format!(
            "Padded input should be {PADDED_FP_LENGTH} bytes, was {}",
            input.len()
        )));
    }
    let (padding, unpadded) = input.split_at(PADDING_LENGTH);
    if !padding.iter().all(|&x| x == 0) {
        return Err(PrecompileError::Other(format!(
            "{PADDING_LENGTH} top bytes of input are not zero",
        )));
    }
    Ok(unpadded.try_into().unwrap())
}
