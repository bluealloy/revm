//! BN128 cryptographic implementations

pub mod constants;

// silence arkworks lint as bn impl will be used as default if both are enabled.
cfg_if::cfg_if! {
    if #[cfg(feature = "bn")]{
        use ark_bn254 as _;
        use ark_ff as _;
        use ark_ec as _;
        use ark_serialize as _;
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "bn")]{
        mod substrate;
        pub use substrate::{g1_point_add, g1_point_mul, pairing_check};
    } else {
        mod arkworks;
        pub use arkworks::{g1_point_add, g1_point_mul, pairing_check};
    }
}
