//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

#[cfg(all(feature = "std", feature = "serde-json"))]
mod eip3155;
mod gas;
mod inspector;
mod noop;
mod prev_inspector;

pub use inspector::*;
pub use prev_inspector::PrevContext;

/// [Inspector] implementations.
pub mod inspectors {
    #[cfg(all(feature = "std", feature = "serde-json"))]
    pub use super::eip3155::TracerEip3155;
    pub use super::gas::GasInspector;
    pub use super::noop::NoOpInspector;
}
