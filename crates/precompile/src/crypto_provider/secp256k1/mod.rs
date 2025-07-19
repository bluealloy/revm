//! secp256k1 cryptographic implementations for the crypto provider

cfg_if::cfg_if! {
    if #[cfg(feature = "secp256k1")] {
        /// Bitcoin secp256k1 backend
        pub mod bitcoin_secp256k1;
        pub use bitcoin_secp256k1::ecrecover;
    } else if #[cfg(feature = "libsecp256k1")] {
        /// Parity libsecp256k1 backend
        pub mod parity_libsecp256k1;
        pub use parity_libsecp256k1::ecrecover;
    } else {
        /// K256 backend
        pub mod k256;
        pub use k256::ecrecover;
    }
}
