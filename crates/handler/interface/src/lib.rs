//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

pub mod frame;
pub mod item_or_result;
pub mod precompile_provider;

pub use frame::Frame;
pub use item_or_result::{FrameInitOrResult, FrameOrResult, ItemOrResult};
pub use precompile_provider::{PrecompileProvider, PrecompileProviderGetter};
