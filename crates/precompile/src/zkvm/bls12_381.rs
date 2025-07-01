//! zkVM implementation of BLS12-381 precompiles.

use crate::PrecompileError;

extern "C" {
    /// zkVM implementation of BLS12-381 G1 point addition.
    ///
    /// # Arguments
    /// * `p1_ptr` - Pointer to first 128-byte G1 point (padded x, y coordinates, 64 bytes each)
    /// * `p2_ptr` - Pointer to second 128-byte G1 point (padded x, y coordinates, 64 bytes each)
    /// * `result_ptr` - Pointer to output buffer for 128-byte result G1 point
    ///
    /// # Returns
    /// * 1 if operation succeeded
    /// * 0 if operation failed (invalid points, etc.)
    fn zkvm_bls12_381_g1_add_impl(p1_ptr: *const u8, p2_ptr: *const u8, result_ptr: *mut u8)
        -> i32;

    /// zkVM implementation of BLS12-381 G1 scalar multiplication.
    ///
    /// # Arguments
    /// * `point_ptr` - Pointer to 128-byte G1 point (padded x, y coordinates, 64 bytes each)
    /// * `scalar_ptr` - Pointer to 32-byte scalar value
    /// * `result_ptr` - Pointer to output buffer for 128-byte result G1 point
    ///
    /// # Returns
    /// * 1 if operation succeeded
    /// * 0 if operation failed (invalid point, etc.)
    fn zkvm_bls12_381_g1_mul_impl(
        point_ptr: *const u8,
        scalar_ptr: *const u8,
        result_ptr: *mut u8,
    ) -> i32;

    /// zkVM implementation of BLS12-381 G2 point addition.
    ///
    /// # Arguments
    /// * `p1_ptr` - Pointer to first 256-byte G2 point (padded coordinates, 64 bytes each)
    /// * `p2_ptr` - Pointer to second 256-byte G2 point (padded coordinates, 64 bytes each)
    /// * `result_ptr` - Pointer to output buffer for 256-byte result G2 point
    ///
    /// # Returns
    /// * 1 if operation succeeded
    /// * 0 if operation failed (invalid points, etc.)
    fn zkvm_bls12_381_g2_add_impl(p1_ptr: *const u8, p2_ptr: *const u8, result_ptr: *mut u8)
        -> i32;

    /// zkVM implementation of BLS12-381 G2 scalar multiplication.
    ///
    /// # Arguments
    /// * `point_ptr` - Pointer to 256-byte G2 point (padded coordinates, 64 bytes each)
    /// * `scalar_ptr` - Pointer to 32-byte scalar value
    /// * `result_ptr` - Pointer to output buffer for 256-byte result G2 point
    ///
    /// # Returns
    /// * 1 if operation succeeded
    /// * 0 if operation failed (invalid point, etc.)
    fn zkvm_bls12_381_g2_mul_impl(
        point_ptr: *const u8,
        scalar_ptr: *const u8,
        result_ptr: *mut u8,
    ) -> i32;

    /// zkVM implementation of BLS12-381 G1 Multi-Scalar Multiplication (MSM).
    ///
    /// # Arguments
    /// * `points_ptr` - Pointer to array of 128-byte G1 points (padded x, y coordinates, 64 bytes each)
    /// * `scalars_ptr` - Pointer to array of 32-byte scalar values
    /// * `num_pairs` - Number of point-scalar pairs
    /// * `result_ptr` - Pointer to output buffer for 128-byte result G1 point
    ///
    /// # Returns
    /// * 1 if operation succeeded
    /// * 0 if operation failed (invalid points, etc.)
    fn zkvm_bls12_381_g1_msm_impl(
        points_ptr: *const u8,
        scalars_ptr: *const u8,
        num_pairs: u32,
        result_ptr: *mut u8,
    ) -> i32;

    /// zkVM implementation of BLS12-381 pairing check.
    ///
    /// # Arguments
    /// * `pairs_ptr` - Pointer to array of G1/G2 point pairs
    ///   Each pair consists of:
    ///   - 128 bytes for G1 point (padded x, y coordinates, 64 bytes each)
    ///   - 256 bytes for G2 point (padded coordinates, 64 bytes each)
    ///   Total: 384 bytes per pair
    /// * `num_pairs` - Number of point pairs
    ///
    /// # Returns
    /// * 1 if pairing check passed (valid input, result equals identity)
    /// * 0 if pairing check failed (valid input, result does not equal identity)
    /// * -1 if invalid input (points not on curve, wrong format, etc.)
    fn zkvm_bls12_381_pairing_impl(pairs_ptr: *const u8, num_pairs: u32) -> i32;

    /// zkVM implementation of BLS12-381 mapping field element to G1 point.
    ///
    /// # Arguments
    /// * `fp_ptr` - Pointer to 64-byte field element (padded)
    /// * `result_ptr` - Pointer to output buffer for 128-byte result G1 point
    ///
    /// # Returns
    /// * 1 if operation succeeded
    /// * 0 if operation failed (invalid field element, etc.)
    fn zkvm_bls12_381_map_fp_to_g1_impl(fp_ptr: *const u8, result_ptr: *mut u8) -> i32;
}

/// Performs G1 point addition using zkVM implementation, matching the backend interface.
#[inline]
pub(super) fn p1_add_affine(
    a_x: &[u8; 48], // FP_LENGTH
    a_y: &[u8; 48],
    b_x: &[u8; 48],
    b_y: &[u8; 48],
) -> Result<[u8; 128], PrecompileError> {
    // PADDED_G1_LENGTH
    // Create 128-byte point representations by padding the coordinates
    let mut p1_bytes = [0u8; 128];
    let mut p2_bytes = [0u8; 128];

    // For BLS12-381, coordinates are padded from 48 bytes to 64 bytes
    // Copy x coordinate with padding
    p1_bytes[16..64].copy_from_slice(a_x); // pad 16 bytes at start
    p1_bytes[80..128].copy_from_slice(a_y); // pad 16 bytes at start

    p2_bytes[16..64].copy_from_slice(b_x);
    p2_bytes[80..128].copy_from_slice(b_y);

    let mut result = [0u8; 128];

    let success = unsafe {
        zkvm_bls12_381_g1_add_impl(p1_bytes.as_ptr(), p2_bytes.as_ptr(), result.as_mut_ptr())
    };

    if success == 1 {
        Ok(result)
    } else {
        Err(PrecompileError::Other(
            "BLS12-381 G1 addition failed".to_string(),
        ))
    }
}

/// Performs G1 scalar multiplication using zkVM implementation.
#[inline]
pub(super) fn g1_point_mul(
    point_bytes: &[u8],
    scalar_bytes: &[u8],
) -> Result<[u8; 128], PrecompileError> {
    let mut result = [0u8; 128];

    let success = unsafe {
        zkvm_bls12_381_g1_mul_impl(
            point_bytes.as_ptr(),
            scalar_bytes.as_ptr(),
            result.as_mut_ptr(),
        )
    };

    if success == 1 {
        Ok(result)
    } else {
        Err(PrecompileError::Other(
            "BLS12-381 G1 multiplication failed".to_string(),
        ))
    }
}

/// Performs G2 point addition using zkVM implementation, matching the backend interface.
#[inline]
pub(super) fn p2_add_affine(
    a_x_0: &[u8; 48], // FP_LENGTH
    a_x_1: &[u8; 48],
    a_y_0: &[u8; 48],
    a_y_1: &[u8; 48],
    b_x_0: &[u8; 48],
    b_x_1: &[u8; 48],
    b_y_0: &[u8; 48],
    b_y_1: &[u8; 48],
) -> Result<[u8; 256], PrecompileError> {
    // PADDED_G2_LENGTH
    // Create 256-byte point representations by padding the coordinates
    let mut p1_bytes = [0u8; 256];
    let mut p2_bytes = [0u8; 256];

    // For BLS12-381 G2, coordinates are padded from 48 bytes to 64 bytes
    // G2 point format: [x0 (64), x1 (64), y0 (64), y1 (64)]
    p1_bytes[16..64].copy_from_slice(a_x_0); // x0 with 16-byte padding
    p1_bytes[80..128].copy_from_slice(a_x_1); // x1 with 16-byte padding
    p1_bytes[144..192].copy_from_slice(a_y_0); // y0 with 16-byte padding
    p1_bytes[208..256].copy_from_slice(a_y_1); // y1 with 16-byte padding

    p2_bytes[16..64].copy_from_slice(b_x_0); // x0 with 16-byte padding
    p2_bytes[80..128].copy_from_slice(b_x_1); // x1 with 16-byte padding
    p2_bytes[144..192].copy_from_slice(b_y_0); // y0 with 16-byte padding
    p2_bytes[208..256].copy_from_slice(b_y_1); // y1 with 16-byte padding

    let mut result = [0u8; 256];

    let success = unsafe {
        zkvm_bls12_381_g2_add_impl(p1_bytes.as_ptr(), p2_bytes.as_ptr(), result.as_mut_ptr())
    };

    if success == 1 {
        Ok(result)
    } else {
        Err(PrecompileError::Other(
            "BLS12-381 G2 addition failed".to_string(),
        ))
    }
}

/// Performs G2 scalar multiplication using zkVM implementation.
#[inline]
pub(super) fn g2_point_mul(
    point_bytes: &[u8],
    scalar_bytes: &[u8],
) -> Result<[u8; 256], PrecompileError> {
    let mut result = [0u8; 256];

    let success = unsafe {
        zkvm_bls12_381_g2_mul_impl(
            point_bytes.as_ptr(),
            scalar_bytes.as_ptr(),
            result.as_mut_ptr(),
        )
    };

    if success == 1 {
        Ok(result)
    } else {
        Err(PrecompileError::Other(
            "BLS12-381 G2 multiplication failed".to_string(),
        ))
    }
}

/// Performs G1 MSM using zkVM implementation, matching the backend interface.
#[inline]
pub(super) fn p1_msm_bytes(
    point_scalar_pairs: &[((&[u8; 48], &[u8; 48]), &[u8; 32])],
) -> Result<[u8; 128], PrecompileError> {
    if point_scalar_pairs.is_empty() {
        return Ok([0u8; 128]); // Return point at infinity
    }

    // Create contiguous buffers for points and scalars
    let mut points_buffer = Vec::with_capacity(point_scalar_pairs.len() * 128);
    let mut scalars_buffer = Vec::with_capacity(point_scalar_pairs.len() * 32);

    for ((x, y), scalar) in point_scalar_pairs {
        // Create 128-byte padded point
        let mut point_bytes = [0u8; 128];
        point_bytes[16..64].copy_from_slice(x); // x with 16-byte padding
        point_bytes[80..128].copy_from_slice(y); // y with 16-byte padding

        points_buffer.extend_from_slice(&point_bytes);
        scalars_buffer.extend_from_slice(scalar);
    }

    let mut result = [0u8; 128];

    let success = unsafe {
        zkvm_bls12_381_g1_msm_impl(
            points_buffer.as_ptr(),
            scalars_buffer.as_ptr(),
            point_scalar_pairs.len() as u32,
            result.as_mut_ptr(),
        )
    };

    if success == 1 {
        Ok(result)
    } else {
        Err(PrecompileError::Other(
            "BLS12-381 G1 MSM failed".to_string(),
        ))
    }
}

/// Performs G2 MSM using zkVM implementation, matching the backend interface.
#[inline]
pub(super) fn p2_msm_bytes(
    point_scalar_pairs: &[((&[u8; 48], &[u8; 48], &[u8; 48], &[u8; 48]), &[u8; 32])],
) -> Result<[u8; 256], PrecompileError> {
    if point_scalar_pairs.is_empty() {
        return Ok([0u8; 256]); // Return point at infinity
    }

    // Create contiguous buffers for points and scalars
    let mut points_buffer = Vec::with_capacity(point_scalar_pairs.len() * 256);
    let mut scalars_buffer = Vec::with_capacity(point_scalar_pairs.len() * 32);

    for ((x0, x1, y0, y1), scalar) in point_scalar_pairs {
        // Create 256-byte padded G2 point
        let mut point_bytes = [0u8; 256];
        point_bytes[16..64].copy_from_slice(x0); // x0 with 16-byte padding
        point_bytes[80..128].copy_from_slice(x1); // x1 with 16-byte padding
        point_bytes[144..192].copy_from_slice(y0); // y0 with 16-byte padding
        point_bytes[208..256].copy_from_slice(y1); // y1 with 16-byte padding

        points_buffer.extend_from_slice(&point_bytes);
        scalars_buffer.extend_from_slice(scalar);
    }

    let mut result = [0u8; 256];

    let success = unsafe {
        zkvm_bls12_381_g2_msm_impl(
            points_buffer.as_ptr(),
            scalars_buffer.as_ptr(),
            point_scalar_pairs.len() as u32,
            result.as_mut_ptr(),
        )
    };

    if success == 1 {
        Ok(result)
    } else {
        Err(PrecompileError::Other(
            "BLS12-381 G2 MSM failed".to_string(),
        ))
    }
}

/// Performs pairing check using zkVM implementation.
#[inline]
pub(super) fn pairing_check(pairs: &[(&[u8], &[u8])]) -> Result<bool, PrecompileError> {
    if pairs.is_empty() {
        return Ok(true);
    }

    // Create a contiguous buffer for all pairs
    // Each pair: 128 bytes (G1) + 256 bytes (G2) = 384 bytes
    let pair_size = 128 + 256; // G1_LENGTH + G2_LENGTH
    let mut buffer = Vec::with_capacity(pairs.len() * pair_size);

    for (g1_bytes, g2_bytes) in pairs {
        // Validate input sizes
        if g1_bytes.len() != 128 {
            return Err(PrecompileError::Other(format!(
                "Invalid G1 point size: expected 128 bytes, got {}",
                g1_bytes.len()
            )));
        }
        if g2_bytes.len() != 256 {
            return Err(PrecompileError::Other(format!(
                "Invalid G2 point size: expected 256 bytes, got {}",
                g2_bytes.len()
            )));
        }

        buffer.extend_from_slice(g1_bytes);
        buffer.extend_from_slice(g2_bytes);
    }

    let result = unsafe { zkvm_bls12_381_pairing_impl(buffer.as_ptr(), pairs.len() as u32) };

    match result {
        1 => Ok(true),  // Pairing passed
        0 => Ok(false), // Pairing failed (valid input)
        -1 => Err(PrecompileError::Other(
            "Invalid BLS12-381 pairing input".to_string(),
        )),
        _ => Err(PrecompileError::Other(format!(
            "Unexpected BLS12-381 pairing result: {}",
            result
        ))),
    }
}

/// Performs pairing check using zkVM implementation with byte-based interface.
#[inline]
pub(super) fn pairing_check_bytes(
    pairs: &[(
        ([u8; 48], [u8; 48]),
        ([u8; 48], [u8; 48], [u8; 48], [u8; 48]),
    )],
) -> Result<bool, PrecompileError> {
    if pairs.is_empty() {
        return Ok(true);
    }

    // Create a contiguous buffer for all pairs
    // Each pair: 128 bytes (G1) + 256 bytes (G2) = 384 bytes
    let mut buffer = Vec::with_capacity(pairs.len() * 384);

    for ((g1_x, g1_y), (g2_x_0, g2_x_1, g2_y_0, g2_y_1)) in pairs {
        // Create padded G1 point (128 bytes)
        let mut g1_padded = [0u8; 128];
        g1_padded[16..64].copy_from_slice(g1_x);
        g1_padded[80..128].copy_from_slice(g1_y);

        // Create padded G2 point (256 bytes)
        let mut g2_padded = [0u8; 256];
        g2_padded[16..64].copy_from_slice(g2_x_0);
        g2_padded[80..128].copy_from_slice(g2_x_1);
        g2_padded[144..192].copy_from_slice(g2_y_0);
        g2_padded[208..256].copy_from_slice(g2_y_1);

        buffer.extend_from_slice(&g1_padded);
        buffer.extend_from_slice(&g2_padded);
    }

    let result = unsafe { zkvm_bls12_381_pairing_impl(buffer.as_ptr(), pairs.len() as u32) };

    match result {
        1 => Ok(true),  // Pairing passed
        0 => Ok(false), // Pairing failed (valid input)
        -1 => Err(PrecompileError::Other(
            "Invalid BLS12-381 pairing input".to_string(),
        )),
        _ => Err(PrecompileError::Other(format!(
            "Unexpected BLS12-381 pairing result: {}",
            result
        ))),
    }
}

/// Maps a field element to a G1 point using zkVM implementation.
#[inline]
pub(super) fn map_fp_to_g1_bytes(fp_bytes: &[u8; 48]) -> Result<[u8; 128], PrecompileError> {
    // Create 64-byte padded field element
    let mut padded_fp = [0u8; 64];
    padded_fp[16..64].copy_from_slice(fp_bytes); // pad 16 bytes at start

    let mut result = [0u8; 128];

    let success =
        unsafe { zkvm_bls12_381_map_fp_to_g1_impl(padded_fp.as_ptr(), result.as_mut_ptr()) };

    if success == 1 {
        Ok(result)
    } else {
        Err(PrecompileError::Other(
            "BLS12-381 map_fp_to_g1 failed".to_string(),
        ))
    }
}

/// Maps a field element to a G2 point using zkVM implementation.
#[inline]
pub(super) fn map_fp2_to_g2_bytes(
    fp2_x: &[u8; 48],
    fp2_y: &[u8; 48],
) -> Result<[u8; 256], PrecompileError> {
    // Create 128-byte padded field element (2 x 64-byte padded FP elements)
    let mut padded_fp2 = [0u8; 128];
    padded_fp2[16..64].copy_from_slice(fp2_x); // x with 16-byte padding
    padded_fp2[80..128].copy_from_slice(fp2_y); // y with 16-byte padding

    let mut result = [0u8; 256];

    let success =
        unsafe { zkvm_bls12_381_map_fp2_to_g2_impl(padded_fp2.as_ptr(), result.as_mut_ptr()) };

    if success == 1 {
        Ok(result)
    } else {
        Err(PrecompileError::Other(
            "BLS12-381 map_fp2_to_g2 failed".to_string(),
        ))
    }
}
