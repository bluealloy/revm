use super::utils::{fp_to_bytes, remove_padding, PADDED_FP_LENGTH};
use blst::{blst_fp_from_bendian, blst_p1_affine, blst_p1_affine_in_g1};
use revm_primitives::{Bytes, PrecompileError};

/// Length of each of the elements in a g1 operation input.
pub(super) const G1_INPUT_ITEM_LENGTH: usize = 128;
/// Output length of a g1 operation.
const G1_OUTPUT_LENGTH: usize = 128;

/// Encodes a G1 point in affine format into a byte slice with padded elements.
pub(super) fn encode_g1_point(input: *const blst_p1_affine) -> Bytes {
    let mut out = vec![0u8; G1_OUTPUT_LENGTH];
    // SAFETY: out comes from fixed length array, input is a blst value.
    unsafe {
        fp_to_bytes(&mut out[..PADDED_FP_LENGTH], &(*input).x);
        fp_to_bytes(&mut out[PADDED_FP_LENGTH..], &(*input).y);
    }
    out.into()
}

/// Extracts a G1 point in Affine format from a 128 byte slice representation.
pub(super) fn extract_g1_input(input: &[u8]) -> Result<blst_p1_affine, PrecompileError> {
    if input.len() != G1_INPUT_ITEM_LENGTH {
        return Err(PrecompileError::Other(format!(
            "Input should be {G1_INPUT_ITEM_LENGTH} bytes, was {}",
            input.len()
        )));
    }

    let input_p0_x = remove_padding(&input[..PADDED_FP_LENGTH])?;
    let input_p0_y = remove_padding(&input[PADDED_FP_LENGTH..G1_INPUT_ITEM_LENGTH])?;

    let mut out = blst_p1_affine::default();
    // SAFETY: input_p0_x and input_p0_y have fixed length, out is a blst value.
    unsafe {
        blst_fp_from_bendian(&mut out.x, input_p0_x.as_ptr());
        blst_fp_from_bendian(&mut out.y, input_p0_y.as_ptr());
    }

    // SAFETY: out is a blst value.
    if unsafe { !blst_p1_affine_in_g1(&out) } {
        return Err(PrecompileError::Other("Element not in G1".to_string()));
    }
    Ok(out)
}
