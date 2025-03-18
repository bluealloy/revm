use super::utils::{fp_from_bendian, fp_to_bytes, remove_padding};
use crate::bls12_381_const::{FP_LENGTH, PADDED_FP_LENGTH, PADDED_G2_LENGTH};
use crate::PrecompileError;
use blst::{blst_fp2, blst_p2_affine, blst_p2_affine_in_g2, blst_p2_affine_on_curve};
use primitives::Bytes;

/// Encodes a G2 point in affine format into byte slice with padded elements.
pub(super) fn encode_g2_point(input: &blst_p2_affine) -> Bytes {
    let mut out = vec![0u8; PADDED_G2_LENGTH];
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

/// Returns a `blst_p2_affine` from the provided byte slices, which represent the x and y
/// affine coordinates of the point.
///
/// - If the x or y coordinate do not represent a canonical field element, an error is returned.
///   See [check_canonical_fp2] for more information.
/// - If the point is not on the curve, an error is returned.
pub(super) fn decode_and_check_g2(
    x1: &[u8; 48],
    x2: &[u8; 48],
    y1: &[u8; 48],
    y2: &[u8; 48],
) -> Result<blst_p2_affine, PrecompileError> {
    let out = blst_p2_affine {
        x: check_canonical_fp2(x1, x2)?,
        y: check_canonical_fp2(y1, y2)?,
    };

    // From EIP-2537:
    //
    // Error cases:
    //
    // * An input is neither a point on the G2 elliptic curve nor the infinity point
    //
    // SAFETY: Out is a blst value.
    if unsafe { !blst_p2_affine_on_curve(&out) } {
        return Err(PrecompileError::Other(
            "Element not on G2 curve".to_string(),
        ));
    }

    Ok(out)
}

/// Checks whether or not the input represents a canonical fp2 field element, returning the field
/// element if successful.
pub(super) fn check_canonical_fp2(
    input_1: &[u8; 48],
    input_2: &[u8; 48],
) -> Result<blst_fp2, PrecompileError> {
    let fp_1 = fp_from_bendian(input_1)?;
    let fp_2 = fp_from_bendian(input_2)?;

    let fp2 = blst_fp2 { fp: [fp_1, fp_2] };

    Ok(fp2)
}
/// Extracts a G2 point in Affine format from a 256 byte slice representation.
///
/// Note: By default, no subgroup checks are performed.
pub(super) fn extract_g2_input(input: &[u8]) -> Result<blst_p2_affine, PrecompileError> {
    _extract_g2_input(input, true)
}
/// Extracts a G2 point in Affine format from a 256 byte slice representation
/// without performing a subgroup check.
///
/// Note: Skipping subgroup checks can introduce security issues.
/// This method should only be called if:
///     - The EIP specifies that no subgroup check should be performed
///     - One can be certain that the point is in the correct subgroup.
pub(super) fn extract_g2_input_no_subgroup_check(
    input: &[u8],
) -> Result<blst_p2_affine, PrecompileError> {
    _extract_g2_input(input, false)
}
/// Extracts a G2 point in Affine format from a 256 byte slice representation.
///
/// **Note**: This function will perform a G2 subgroup check if `subgroup_check` is set to `true`.
fn _extract_g2_input(
    input: &[u8],
    subgroup_check: bool,
) -> Result<blst_p2_affine, PrecompileError> {
    if input.len() != PADDED_G2_LENGTH {
        return Err(PrecompileError::Other(format!(
            "Input should be {PADDED_G2_LENGTH} bytes, was {}",
            input.len()
        )));
    }

    let mut input_fps = [&[0; FP_LENGTH]; 4];
    for i in 0..4 {
        input_fps[i] = remove_padding(&input[i * PADDED_FP_LENGTH..(i + 1) * PADDED_FP_LENGTH])?;
    }

    let out = decode_and_check_g2(input_fps[0], input_fps[1], input_fps[2], input_fps[3])?;

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
    }
    Ok(out)
}
