//! BN128 cryptographic implementations

pub mod constants;

cfg_if::cfg_if! {
    if #[cfg(feature = "bn")]{
        mod substrate;
        pub use substrate::{g1_point_add, g1_point_mul, pairing_check};
    } else {
        mod arkworks;
        pub use arkworks::{g1_point_add, g1_point_mul, pairing_check};
    }
}