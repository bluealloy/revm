//! C-KZG implementation for KZG operations

use c_kzg::{Bytes32, Bytes48};

/// Verify KZG proof using C-KZG library.
pub fn verify_kzg_proof(
    commitment: &[u8; 48],
    z: &[u8; 32],
    y: &[u8; 32],
    proof: &[u8; 48],
) -> bool {
    let kzg_settings = c_kzg::ethereum_kzg_settings(8);
    kzg_settings
        .verify_kzg_proof(as_bytes48(commitment), as_bytes32(z), as_bytes32(y), as_bytes48(proof))
        .unwrap_or(false)
}

/// Convert a slice to a 32 byte big endian array.
#[inline]
#[track_caller]
fn as_bytes32(bytes: &[u8; 32]) -> &Bytes32 {
    // SAFETY: `#[repr(C)] Bytes32([u8; 32])`
    unsafe { &*bytes.as_ptr().cast() }
}

/// Convert a slice to a 48 byte big endian array.
#[inline]
#[track_caller]
fn as_bytes48(bytes: &[u8; 48]) -> &Bytes48 {
    // SAFETY: `#[repr(C)] Bytes48([u8; 48])`
    unsafe { &*bytes.as_ptr().cast() }
}