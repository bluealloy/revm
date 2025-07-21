//! secp256k1 cryptographic implementations

pub mod constants;

cfg_if::cfg_if! {
    if #[cfg(feature = "secp256k1")]{
        mod bitcoin_secp256k1;
        pub use bitcoin_secp256k1::ecrecover;
    } else if #[cfg(feature = "libsecp256k1")]{
        mod parity_libsecp256k1;
        pub use parity_libsecp256k1::ecrecover;
    } else {
        mod k256;
        pub use k256::ecrecover;
    }
}
