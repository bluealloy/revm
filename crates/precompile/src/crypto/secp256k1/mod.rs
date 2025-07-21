//! secp256k1 cryptographic implementations

pub mod constants;

// Select and silence unused dependencies based on feature selection
cfg_if::cfg_if! {
    if #[cfg(feature = "secp256k1")]{
        mod bitcoin_secp256k1;
        pub use bitcoin_secp256k1::ecrecover;

        // k256 is unused when secp256k1 is selected
        use k256 as _;

        // libsecp256k1 is also unused when secp256k1 is selected
        #[cfg(feature = "libsecp256k1")]
        use libsecp256k1 as _;
    } else if #[cfg(feature = "libsecp256k1")]{
        mod parity_libsecp256k1;
        pub use parity_libsecp256k1::ecrecover;

        // k256 is unused when libsecp256k1 is selected
        use k256 as _;
    } else {
        mod k256;
        pub use k256::ecrecover;
    }
}
