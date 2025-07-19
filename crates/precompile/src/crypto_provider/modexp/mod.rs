//! Modexp cryptographic implementations for the crypto provider

cfg_if::cfg_if! {
    if #[cfg(feature = "gmp")] {
        /// GMP backend for modexp operations
        pub mod gmp;
        pub use gmp::modexp;
    } else {
        /// Aurora engine backend for modexp operations
        pub mod aurora;
        pub use aurora::modexp;
    }
}
