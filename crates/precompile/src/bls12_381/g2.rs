use super::utils::{fp_to_bytes, remove_padding, FP_LENGTH, PADDED_FP_LENGTH};
use blst::{blst_fp_from_bendian, blst_p2_affine, blst_p2_affine_in_g2, blst_p2_affine_on_curve};
use revm_primitives::{Bytes, PrecompileError};

/// Length of each of the elements in a g2 operation input.
pub(super) const G2_INPUT_ITEM_LENGTH: usize = 256;
/// Output length of a g2 operation.
const G2_OUTPUT_LENGTH: usize = 256;

/// Encodes a G2 point in affine format into a byte slice with padded elements.
pub(super) fn encode_g2_point(input: &blst_p2_affine) -> Bytes {
    let mut out = vec![0u8; G2_OUTPUT_LENGTH];
    fp_to_bytes(&mut out[..PADDED_FP_LENGTH], &input.x.fp[0]);
    fp_to_bytes(
        &mut out[PADDED_FP_LENGTH..2 * PADDED_FP_LENGTH],
        &input.x.fp[1],
    );
    fp_to_bytes(
        &mut out[2 * PADDED_FP_LENGTH..3 * PADDED_FP_LENGTH],
        &input.y.fp[0],
    );
    fp_to_bytes(
        &mut out[3 * PADDED_FP_LENGTH..4 * PADDED_FP_LENGTH],
        &input.y.fp[1],
    );
    out.into()
}

/// Extracts a G2 point in Affine format from a 256 byte slice representation.
///
/// NOTE: This function will perform a G2 subgroup check if `subgroup_check` is set to `true`.
pub(super) fn extract_g2_input(
    input: &[u8],
    subgroup_check: bool,
) -> Result<blst_p2_affine, PrecompileError> {
    if input.len() != G2_INPUT_ITEM_LENGTH {
        return Err(PrecompileError::Other(format!(
            "Input should be {G2_INPUT_ITEM_LENGTH} bytes, was {}",
            input.len()
        )));
    }

    let mut input_fps: [&[u8; FP_LENGTH]; 4] = [&[0; FP_LENGTH]; 4];
    for i in 0..4 {
        input_fps[i] = remove_padding(&input[i * PADDED_FP_LENGTH..(i + 1) * PADDED_FP_LENGTH])?;
    }

    let mut out = blst_p2_affine::default();
    // SAFETY: items in fps have fixed length, out is a blst value.
    unsafe {
        blst_fp_from_bendian(&mut out.x.fp[0], input_fps[0].as_ptr());
        blst_fp_from_bendian(&mut out.x.fp[1], input_fps[1].as_ptr());
        blst_fp_from_bendian(&mut out.y.fp[0], input_fps[2].as_ptr());
        blst_fp_from_bendian(&mut out.y.fp[1], input_fps[3].as_ptr());
    }

    if subgroup_check {
        // NB: Subgroup checks
        //
        // Scalar multiplications, MSMs and pairings MUST perform a subgroup check.
        //
        // Implementations SHOULD use the optimized subgroup check method:
        //
        // https://eips.ethereum.org/assets/eip-2537/fast_subgroup_checks
        //
        // On any input that fail the subgroup check, the precompile MUST return an error.
        //
        // As endomorphism acceleration requires input on the correct subgroup, implementers MAY
        // use endomorphism acceleration.
        if unsafe { !blst_p2_affine_in_g2(&out) } {
            return Err(PrecompileError::Other("Element not in G2".to_string()));
        }
    } else {
        // From EIP-2537:
        //
        // Error cases:
        //
        // * An input is neither a point on the G2 elliptic curve nor the infinity point
        //
        // NB: There is no subgroup check for the G2 addition precompile.
        //
        // We use blst_p2_affine_on_curve instead of blst_p2_affine_in_g2 because the latter performs
        // the subgroup check.
        //
        // SAFETY: out is a blst value.
        if unsafe { !blst_p2_affine_on_curve(&out) } {
            return Err(PrecompileError::Other(
                "Element not on G2 curve".to_string(),
            ));
        }
    }

    Ok(out)
}
