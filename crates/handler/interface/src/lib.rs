//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

pub mod frame;
pub mod precompile_provider;
pub mod util;

pub use frame::Frame;
pub use precompile_provider::{PrecompileProvider, PrecompileProviderGetter};
pub use util::FrameOrResult;

/// Init and floor gas from transaction
#[derive(Clone, Copy, Debug, Default)]
pub struct InitialAndFloorGas {
    /// Initial gas for transaction.
    pub initial_gas: u64,
    /// If transaction is a Call and Prague is enabled
    /// floor_gas is at least amount of gas that is going to be spent.
    pub floor_gas: u64,
}
