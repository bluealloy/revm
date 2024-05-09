use blst::{blst_fp_from_bendian, blst_p2_affine, blst_p2_affine_in_g2};
use revm_primitives::PrecompileError;

use super::utils::{fp_to_bytes, remove_padding, FP_LENGTH, PADDED_FP_LENGTH};

/// Length of each of the elements in a g2 operation input.
pub(super) const G2_INPUT_ITEM_LENGTH: usize = 256;
/// Output length of a g2 operation.
pub(super) const G2_OUTPUT_LENGTH: usize = 256;

/// Encodes a G2 point in affine format into a byte slice with padded elements.
pub(super) fn encode_g2_point(out: &mut [u8], input: *const blst_p2_affine) {
    // SAFETY: out comes from fixed length array, input is a blst value.
    unsafe {
        fp_to_bytes(&mut out[..PADDED_FP_LENGTH], &(*input).x.fp[0]);
        fp_to_bytes(
            &mut out[PADDED_FP_LENGTH..2 * PADDED_FP_LENGTH],
            &(*input).x.fp[1],
        );
        fp_to_bytes(
            &mut out[2 * PADDED_FP_LENGTH..3 * PADDED_FP_LENGTH],
            &(*input).y.fp[0],
        );
        fp_to_bytes(
            &mut out[3 * PADDED_FP_LENGTH..4 * PADDED_FP_LENGTH],
            &(*input).y.fp[1],
        );
    }
}

/// Extracts a G2 point in Affine format from a 256 byte slice representation.
pub(super) fn extract_g2_input(
    out: *mut blst_p2_affine,
    input: &[u8],
) -> Result<*mut blst_p2_affine, PrecompileError> {
    if input.len() != G2_INPUT_ITEM_LENGTH {
        return Err(PrecompileError::Other(format!(
            "Input should be {G2_INPUT_ITEM_LENGTH} bits, was {}",
            input.len()
        )));
    }

    let mut input_fps: [[u8; FP_LENGTH]; 4] = [[0; FP_LENGTH]; 4];
    for i in 0..4 {
        input_fps[i] = remove_padding(&input[i * PADDED_FP_LENGTH..(i + 1) * PADDED_FP_LENGTH])?;
    }

    // SAFETY: items in fps have fixed length, out is a blst value.
    unsafe {
        blst_fp_from_bendian(&mut (*out).x.fp[0], input_fps[0].as_ptr());
        blst_fp_from_bendian(&mut (*out).x.fp[1], input_fps[1].as_ptr());
        blst_fp_from_bendian(&mut (*out).y.fp[0], input_fps[2].as_ptr());
        blst_fp_from_bendian(&mut (*out).y.fp[1], input_fps[3].as_ptr());
    }
    // SAFETY: out is a blst value.
    unsafe {
        if !blst_p2_affine_in_g2(out) {
            return Err(PrecompileError::Other("Element not in G2".to_string()));
        }
    }
    Ok(out)
}
