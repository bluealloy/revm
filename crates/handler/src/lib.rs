//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

// Mainnet related handlers.

pub mod execution;
mod frame;
mod frame_data;
pub mod handler;
pub mod instructions;
pub mod post_execution;
pub mod pre_execution;
mod precompile_provider;
pub mod validation;

// Public exports
pub use frame::{return_create, return_eofcreate, CtxTraitDbError, EthFrame, EthFrameContext};
pub use frame_data::{FrameData, FrameResult};
pub use handler::{EthContext, EthError, EthHandler, MainnetHandler};
pub use precompile_provider::{EthPrecompileProvider, PrecompileProvider};
