//! KZG-RS implementation for KZG operations

use kzg_rs::{Bytes32, Bytes48, KzgProof};

/// Verify KZG proof using KZG-RS library.
pub fn verify_kzg_proof(
    commitment: &[u8; 48],
    z: &[u8; 32],
    y: &[u8; 32],
    proof: &[u8; 48],
) -> bool {
    let env = kzg_rs::EnvKzgSettings::default();
    let kzg_settings = env.get();
    KzgProof::verify_kzg_proof(
        as_bytes48(commitment),
        as_bytes32(z),
        as_bytes32(y),
        as_bytes48(proof),
        kzg_settings,
    )
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
