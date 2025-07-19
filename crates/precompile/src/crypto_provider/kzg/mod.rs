//! KZG cryptographic implementations for the crypto provider

cfg_if::cfg_if! {
    if #[cfg(feature = "c-kzg")] {
        use ::c_kzg::{Bytes32, Bytes48};

        /// C-KZG backend for KZG operations
        pub mod c_kzg;
        pub use c_kzg::verify_kzg_proof;
    } else if #[cfg(feature = "kzg-rs")] {
        use ::kzg_rs::{Bytes32, Bytes48};

        /// KZG-RS backend for KZG operations
        pub mod kzg_rs;
        pub use kzg_rs::verify_kzg_proof;
    }
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
