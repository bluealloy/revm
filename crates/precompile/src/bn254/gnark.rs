//! BN254 precompile using gnark-crypto Go library via FFI
//!
//! This module provides BN254 elliptic curve operations using the gnark-crypto
//! library from Go, accessed through FFI bindings. The gnark library is highly
//! optimized and widely used in zkSNARK applications.

use super::{G1_LEN, G2_LEN, SCALAR_LEN};
use crate::PrecompileError;
use std::vec::Vec;

/// Performs point addition on two G1 points using gnark-crypto.
///
/// # Parameters
///
/// - `p1_bytes`: First G1 point as 64 bytes (x, y coordinates, 32 bytes each, big-endian)
/// - `p2_bytes`: Second G1 point as 64 bytes
///
/// # Returns
///
/// - `Ok([u8; 64])`: The sum of the two points
/// - `Err(PrecompileError)`: If points are invalid or operation fails
///
/// # Panics
///
/// Panics if input slices are not exactly 64 bytes each.
#[inline]
pub(crate) fn g1_point_add(
    p1_bytes: &[u8],
    p2_bytes: &[u8],
) -> Result<[u8; 64], PrecompileError> {
    assert_eq!(
        p1_bytes.len(),
        G1_LEN,
        "p1 must be {G1_LEN} bytes, got {}",
        p1_bytes.len()
    );
    assert_eq!(
        p2_bytes.len(),
        G1_LEN,
        "p2 must be {G1_LEN} bytes, got {}",
        p2_bytes.len()
    );

    let mut output = [0u8; 64];

    let result = unsafe {
        revm_gnark_bn254::gnark_bn254_g1_add(
            p1_bytes.as_ptr(),
            p2_bytes.as_ptr(),
            output.as_mut_ptr(),
        )
    };

    match result {
        0 => Ok(output),
        -1 => Err(PrecompileError::Bn254AffineGFailedToCreate),
        _ => Err(PrecompileError::other(format!(
            "gnark g1 add failed with code {}",
            result
        ))),
    }
}

/// Performs scalar multiplication on a G1 point using gnark-crypto.
///
/// # Parameters
///
/// - `point_bytes`: G1 point as 64 bytes (x, y coordinates, 32 bytes each, big-endian)
/// - `scalar_bytes`: Scalar as 32 bytes (big-endian)
///
/// # Returns
///
/// - `Ok([u8; 64])`: The result of scalar multiplication
/// - `Err(PrecompileError)`: If point is invalid or operation fails
///
/// # Panics
///
/// Panics if input slices are not the correct length.
#[inline]
pub(crate) fn g1_point_mul(
    point_bytes: &[u8],
    scalar_bytes: &[u8],
) -> Result<[u8; 64], PrecompileError> {
    assert_eq!(
        point_bytes.len(),
        G1_LEN,
        "point must be {G1_LEN} bytes, got {}",
        point_bytes.len()
    );
    assert_eq!(
        scalar_bytes.len(),
        SCALAR_LEN,
        "scalar must be {SCALAR_LEN} bytes, got {}",
        scalar_bytes.len()
    );

    let mut output = [0u8; 64];

    let result = unsafe {
        revm_gnark_bn254::gnark_bn254_g1_mul(
            point_bytes.as_ptr(),
            scalar_bytes.as_ptr(),
            output.as_mut_ptr(),
        )
    };

    match result {
        0 => Ok(output),
        -1 => Err(PrecompileError::Bn254AffineGFailedToCreate),
        _ => Err(PrecompileError::other(format!(
            "gnark g1 mul failed with code {}",
            result
        ))),
    }
}

/// Performs a pairing check on G1/G2 point pairs using gnark-crypto.
///
/// This function verifies that the pairing product of the given point pairs
/// equals the identity element in GT (the target group). This is used in
/// signature verification schemes and zkSNARK verification.
///
/// # Parameters
///
/// - `pairs`: Slice of (G1, G2) point pairs
///   - Each G1 point is 64 bytes: x (32) | y (32), big-endian
///   - Each G2 point is 128 bytes: x_imag (32) | x_real (32) | y_imag (32) | y_real (32), big-endian
///
/// # Returns
///
/// - `Ok(true)`: Pairing check succeeded (product is identity)
/// - `Ok(false)`: Pairing check failed (product is not identity)
/// - `Err(PrecompileError)`: If points are invalid or operation fails
///
/// # Note
///
/// If the input is empty (no pairs), this function returns `Ok(true)`, which
/// matches the behavior of other BN254 implementations in REVM.
#[inline]
pub(crate) fn pairing_check(pairs: &[(&[u8], &[u8])]) -> Result<bool, PrecompileError> {
    // Empty input returns true (identity element)
    if pairs.is_empty() {
        return Ok(true);
    }

    // Flatten pairs into a single buffer for the FFI call
    // Format: [G1_1 | G2_1 | G1_2 | G2_2 | ...]
    let mut pairs_data = Vec::with_capacity(pairs.len() * (G1_LEN + G2_LEN));

    for (g1_bytes, g2_bytes) in pairs {
        assert_eq!(
            g1_bytes.len(),
            G1_LEN,
            "G1 point must be {G1_LEN} bytes, got {}",
            g1_bytes.len()
        );
        assert_eq!(
            g2_bytes.len(),
            G2_LEN,
            "G2 point must be {G2_LEN} bytes, got {}",
            g2_bytes.len()
        );

        pairs_data.extend_from_slice(g1_bytes);
        pairs_data.extend_from_slice(g2_bytes);
    }

    let mut result: u8 = 0;

    let ret_code = unsafe {
        revm_gnark_bn254::gnark_bn254_pairing_check(
            pairs_data.as_ptr(),
            pairs.len() as i32,
            &mut result as *mut u8,
        )
    };

    match ret_code {
        0 => Ok(result == 1),
        -1 => Err(PrecompileError::Bn254AffineGFailedToCreate),
        -2 => Err(PrecompileError::Bn254AffineGFailedToCreate),
        -3 => Err(PrecompileError::other("gnark pairing check failed")),
        _ => Err(PrecompileError::other(format!(
            "gnark pairing check failed with code {}",
            ret_code
        ))),
    }
}
