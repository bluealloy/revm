use blst::{blst_fp_from_bendian, blst_p1_affine, blst_p1_affine_in_g1};
use revm_primitives::PrecompileError;

use super::utils::{fp_to_bytes, remove_padding, PADDED_FP_LENGTH};

/// Length of each of the elements in a g1 operation input.
pub(super) const G1_INPUT_ITEM_LENGTH: usize = 128;
/// Output length of a g1 operation.
pub(super) const G1_OUTPUT_LENGTH: usize = 128;

/// Encodes a G1 point in affine format into a byte slice with padded elements.
pub(super) fn encode_g1_point(out: &mut [u8], input: *const blst_p1_affine) {
    // SAFETY: out comes from fixed length array, x and y are blst values.
    unsafe {
        fp_to_bytes(&mut out[..PADDED_FP_LENGTH], &(*input).x);
        fp_to_bytes(&mut out[PADDED_FP_LENGTH..], &(*input).y);
    }
}

/// Extracts a G1 point in Affine format from a 128 byte slice representation.
pub(super) fn extract_g1_input(
    out: *mut blst_p1_affine,
    input: &[u8],
) -> Result<*mut blst_p1_affine, PrecompileError> {
    if input.len() != G1_INPUT_ITEM_LENGTH {
        return Err(PrecompileError::Other(format!(
            "Input should be {G1_INPUT_ITEM_LENGTH} bits, was {}",
            input.len()
        )));
    }

    let input_p0_x = remove_padding(&input[..PADDED_FP_LENGTH])?;
    let input_p0_y = remove_padding(&input[PADDED_FP_LENGTH..G1_INPUT_ITEM_LENGTH])?;

    // SAFETY: input_p0_x and input_p0_y have fixed length, x and y are blst values.
    unsafe {
        blst_fp_from_bendian(&mut (*out).x, input_p0_x.as_ptr());
        blst_fp_from_bendian(&mut (*out).y, input_p0_y.as_ptr());
    }
    // SAFETY: out is a blst value.
    unsafe {
        if !blst_p1_affine_in_g1(out) {
            return Err(PrecompileError::Other("Element not in G1".to_string()));
        }
    }
    Ok(out)
}
