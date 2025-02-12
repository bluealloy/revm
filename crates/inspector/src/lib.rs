//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

// #[cfg(not(feature = "std"))]
// extern crate alloc as std;

#[cfg(all(feature = "std", feature = "serde-json"))]
mod eip3155;
mod gas;
mod inspect;
mod inspector;
mod mainnet_inspect;
mod noop;
mod traits;

/// Inspector implementations.
pub mod inspectors {
    #[cfg(all(feature = "std", feature = "serde-json"))]
    pub use super::eip3155::TracerEip3155;
    pub use super::gas::GasInspector;
}

pub use inspect::{InspectCommitEvm, InspectEvm};
pub use inspector::*;
pub use noop::NoOpInspector;
pub use traits::*;
