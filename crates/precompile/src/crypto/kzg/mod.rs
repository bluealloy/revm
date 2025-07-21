//! KZG (Kate-Zaverucha-Goldberg) point evaluation

#[cfg(all(feature = "c-kzg", feature = "kzg-rs"))]
// silence kzg-rs lint as c-kzg will be used as default if both are enabled.
use kzg_rs as _;

cfg_if::cfg_if! {
    if #[cfg(feature = "c-kzg")] {
        use c_kzg::{Bytes32, Bytes48};
    } else if #[cfg(feature = "kzg-rs")] {
        use kzg_rs::{Bytes32, Bytes48, KzgProof};
    }
}

/// Verify KZG proof.
#[inline]
pub fn verify_kzg_proof(
    commitment: &[u8; 48],
    z: &[u8; 32],
    y: &[u8; 32],
    proof: &[u8; 48],
) -> bool {
    cfg_if::cfg_if! {
        if #[cfg(feature = "c-kzg")] {
            let kzg_settings = c_kzg::ethereum_kzg_settings(8);
            kzg_settings.verify_kzg_proof(as_bytes48(commitment), as_bytes32(z), as_bytes32(y), as_bytes48(proof)).unwrap_or(false)
        } else if #[cfg(feature = "kzg-rs")] {
            let env = kzg_rs::EnvKzgSettings::default();
            let kzg_settings = env.get();
            KzgProof::verify_kzg_proof(as_bytes48(commitment), as_bytes32(z), as_bytes32(y), as_bytes48(proof), kzg_settings).unwrap_or(false)
        }
    }
}

/// Convert a slice to an array of a specific size.
#[inline]
#[track_caller]
fn as_array<const N: usize>(bytes: &[u8]) -> &[u8; N] {
    bytes.try_into().expect("slice with incorrect length")
}

/// Convert a slice to a 32 byte big endian array.
#[inline]
#[track_caller]
fn as_bytes32(bytes: &[u8]) -> &Bytes32 {
    // SAFETY: `#[repr(C)] Bytes32([u8; 32])`
    unsafe { &*as_array::<32>(bytes).as_ptr().cast() }
}

/// Convert a slice to a 48 byte big endian array.
#[inline]
#[track_caller]
fn as_bytes48(bytes: &[u8]) -> &Bytes48 {
    // SAFETY: `#[repr(C)] Bytes48([u8; 48])`
    unsafe { &*as_array::<48>(bytes).as_ptr().cast() }
}
