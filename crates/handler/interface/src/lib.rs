//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

pub mod execution;
pub mod frame;
pub mod handler;
pub mod post_execution;
pub mod pre_execution;
pub mod precompile_provider;
pub mod util;
pub mod validation;

pub use execution::ExecutionHandler;
pub use frame::Frame;
pub use handler::Handler;
pub use post_execution::PostExecutionHandler;
pub use pre_execution::PreExecutionHandler;
pub use precompile_provider::{PrecompileProvider, PrecompileProviderGetter};
pub use util::FrameOrResultGen;
pub use validation::{InitialAndFloorGas, ValidationHandler};
