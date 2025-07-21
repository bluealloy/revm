//! secp256k1 cryptographic implementations
//!
//! Depending on enabled features, it will use different implementations of `ecrecover`:
//! * [`k256`](https://crates.io/crates/k256) - uses maintained pure rust lib `k256`, it is perfect use for no_std environments.
//! * [`secp256k1`](https://crates.io/crates/secp256k1) - uses `bitcoin_secp256k1` lib, it is a C implementation of secp256k1 used in bitcoin core.
//!   It is faster than k256 and enabled by default and in std environment.
//! * [`libsecp256k1`](https://crates.io/crates/libsecp256k1) - is made from parity in pure rust, it is alternative for k256.
//!
//! Order of preference is `secp256k1` -> `k256` -> `libsecp256k1`. Where if no features are enabled, it will use `k256`.

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
