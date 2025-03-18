use super::utils::{fp_from_bendian, fp_to_bytes, remove_padding};
use crate::bls12_381_const::{PADDED_FP_LENGTH, PADDED_G1_LENGTH};
use crate::PrecompileError;
use blst::{blst_p1_affine, blst_p1_affine_in_g1, blst_p1_affine_on_curve};
use primitives::Bytes;

/// Encodes a G1 point in affine format into byte slice with padded elements.
pub(super) fn encode_g1_point(input: *const blst_p1_affine) -> Bytes {
    let mut out = vec![0u8; PADDED_G1_LENGTH];
    // SAFETY: Out comes from fixed length array, input is a blst value.
    unsafe {
        fp_to_bytes(&mut out[..PADDED_FP_LENGTH], &(*input).x);
        fp_to_bytes(&mut out[PADDED_FP_LENGTH..], &(*input).y);
    }
    out.into()
}

/// Returns a `blst_p1_affine` from the provided byte slices, which represent the x and y
/// affine coordinates of the point.
///
/// - If the x or y coordinate do not represent a canonical field element, an error is returned.
///   See [fp_from_bendian] for more information.
/// - If the point is not on the curve, an error is returned.
pub(super) fn decode_and_check_g1(
    p0_x: &[u8; 48],
    p0_y: &[u8; 48],
) -> Result<blst_p1_affine, PrecompileError> {
    let out = blst_p1_affine {
        x: fp_from_bendian(p0_x)?,
        y: fp_from_bendian(p0_y)?,
    };

    // From EIP-2537:
    //
    // Error cases:
    //
    // * An input is neither a point on the G1 elliptic curve nor the infinity point
    //
    // SAFETY: Out is a blst value.
    if unsafe { !blst_p1_affine_on_curve(&out) } {
        return Err(PrecompileError::Other(
            "Element not on G1 curve".to_string(),
        ));
    }

    Ok(out)
}

/// Extracts a G1 point in Affine format from a 128 byte slice representation.
///
/// Note: By default, subgroup checks are performed.
pub(super) fn extract_g1_input(input: &[u8]) -> Result<blst_p1_affine, PrecompileError> {
    _extract_g1_input(input, true)
}
/// Extracts a G1 point in Affine format from a 128 byte slice representation.
/// without performing a subgroup check.
///
/// Note: Skipping subgroup checks can introduce security issues.
/// This method should only be called if:
///     - The EIP specifies that no subgroup check should be performed
///     - One can be certain that the point is in the correct subgroup.
pub(super) fn extract_g1_input_no_subgroup_check(
    input: &[u8],
) -> Result<blst_p1_affine, PrecompileError> {
    _extract_g1_input(input, false)
}
/// Extracts a G1 point in Affine format from a 128 byte slice representation.
///
/// **Note**: This function will perform a G1 subgroup check if `subgroup_check` is set to `true`.
fn _extract_g1_input(
    input: &[u8],
    subgroup_check: bool,
) -> Result<blst_p1_affine, PrecompileError> {
    if input.len() != PADDED_G1_LENGTH {
        return Err(PrecompileError::Other(format!(
            "Input should be {PADDED_G1_LENGTH} bytes, was {}",
            input.len()
        )));
    }

    let input_p0_x = remove_padding(&input[..PADDED_FP_LENGTH])?;
    let input_p0_y = remove_padding(&input[PADDED_FP_LENGTH..PADDED_G1_LENGTH])?;
    let out = decode_and_check_g1(input_p0_x, input_p0_y)?;

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
        if unsafe { !blst_p1_affine_in_g1(&out) } {
            return Err(PrecompileError::Other("Element not in G1".to_string()));
        }
    }
    Ok(out)
}
