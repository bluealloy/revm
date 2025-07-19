//! KZG cryptographic implementations for the crypto provider

cfg_if::cfg_if! {
    if #[cfg(feature = "c-kzg")] {
        /// C-KZG backend for KZG operations
        pub mod c_kzg;
        pub use c_kzg::verify_kzg_proof;
    } else if #[cfg(feature = "kzg-rs")] {
        /// KZG-RS backend for KZG operations
        pub mod kzg_rs;
        pub use kzg_rs::verify_kzg_proof;
    }
}