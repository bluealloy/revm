//! zkVM implementation of BN128 precompiles.

use crate::PrecompileError;

extern "C" {
    /// zkVM implementation of BN128 G1 point addition.
    ///
    /// # Arguments
    /// * `p1_ptr` - Pointer to first 64-byte G1 point (x, y coordinates, 32 bytes each)
    /// * `p2_ptr` - Pointer to second 64-byte G1 point (x, y coordinates, 32 bytes each)
    /// * `result_ptr` - Pointer to output buffer for 64-byte result G1 point
    ///
    /// # Returns
    /// * 1 if operation succeeded
    /// * 0 if operation failed (invalid points, etc.)
    fn zkvm_bn128_add_impl(p1_ptr: *const u8, p2_ptr: *const u8, result_ptr: *mut u8) -> i32;

    /// zkVM implementation of BN128 G1 scalar multiplication.
    ///
    /// # Arguments
    /// * `point_ptr` - Pointer to 64-byte G1 point (x, y coordinates, 32 bytes each)
    /// * `scalar_ptr` - Pointer to 32-byte scalar value
    /// * `result_ptr` - Pointer to output buffer for 64-byte result G1 point
    ///
    /// # Returns
    /// * 1 if operation succeeded
    /// * 0 if operation failed (invalid point, etc.)
    fn zkvm_bn128_mul_impl(point_ptr: *const u8, scalar_ptr: *const u8, result_ptr: *mut u8)
        -> i32;

    /// zkVM implementation of BN128 pairing check.
    ///
    /// # Arguments
    /// * `pairs_ptr` - Pointer to array of G1/G2 point pairs
    ///   Each pair consists of:
    ///   - 64 bytes for G1 point (x, y coordinates, 32 bytes each)
    ///   - 128 bytes for G2 point (x0, x1, y0, y1 coordinates, 32 bytes each)
    ///   Total: 192 bytes per pair
    /// * `num_pairs` - Number of point pairs
    ///
    /// # Returns
    /// * 1 if pairing check passed (result equals identity)
    /// * 0 if pairing check failed or invalid input
    fn zkvm_bn128_pairing_impl(pairs_ptr: *const u8, num_pairs: u32) -> i32;
}

/// Performs point addition on two G1 points using zkVM implementation.
#[inline]
pub(super) fn g1_point_add(p1_bytes: &[u8], p2_bytes: &[u8]) -> Result<[u8; 64], PrecompileError> {
    let mut result = [0u8; 64];

    let success =
        unsafe { zkvm_bn128_add_impl(p1_bytes.as_ptr(), p2_bytes.as_ptr(), result.as_mut_ptr()) };

    if success == 1 {
        Ok(result)
    } else {
        Err(PrecompileError::Bn128AffineGFailedToCreate)
    }
}

/// Performs G1 scalar multiplication using zkVM implementation.
#[inline]
pub(super) fn g1_point_mul(
    point_bytes: &[u8],
    scalar_bytes: &[u8],
) -> Result<[u8; 64], PrecompileError> {
    let mut result = [0u8; 64];

    let success = unsafe {
        zkvm_bn128_mul_impl(
            point_bytes.as_ptr(),
            scalar_bytes.as_ptr(),
            result.as_mut_ptr(),
        )
    };

    if success == 1 {
        Ok(result)
    } else {
        Err(PrecompileError::Bn128AffineGFailedToCreate)
    }
}

/// Performs pairing check using zkVM implementation.
#[inline]
pub(super) fn pairing_check(pairs: &[(&[u8], &[u8])]) -> Result<bool, PrecompileError> {
    if pairs.is_empty() {
        return Ok(true);
    }

    // Create a contiguous buffer for all pairs
    // Each pair: 64 bytes (G1) + 128 bytes (G2) = 192 bytes
    let pair_size = 64 + 128; // G1_LEN + G2_LEN
    let mut buffer = Vec::with_capacity(pairs.len() * pair_size);

    for (g1_bytes, g2_bytes) in pairs {
        // Validate input sizes
        if g1_bytes.len() != 64 {
            return Err(PrecompileError::Other(format!(
                "Invalid G1 point size: expected 64 bytes, got {}",
                g1_bytes.len()
            )));
        }
        if g2_bytes.len() != 128 {
            return Err(PrecompileError::Other(format!(
                "Invalid G2 point size: expected 128 bytes, got {}",
                g2_bytes.len()
            )));
        }

        buffer.extend_from_slice(g1_bytes);
        buffer.extend_from_slice(g2_bytes);
    }

    let success = unsafe { zkvm_bn128_pairing_impl(buffer.as_ptr(), pairs.len() as u32) };

    Ok(success == 1)
}
