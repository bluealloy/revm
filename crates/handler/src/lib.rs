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
pub mod inspector;
pub mod instructions;
mod item_or_result;
mod mainnet_handler;
pub mod noop;
pub mod post_execution;
pub mod pre_execution;
mod precompile_provider;
pub mod validation;

// Public exports
pub use frame::{return_create, return_eofcreate, CtxTraitDbError, EthFrame, Frame};
pub use frame_data::{FrameData, FrameResult};
pub use handler::{inspect_instructions, EthHandler, EthTraitError, EvmTrait};
pub use item_or_result::{FrameInitOrResult, FrameOrResult, ItemOrResult};
pub use mainnet_handler::MainnetHandler;
pub use precompile_provider::{EthPrecompiles, PrecompileProvider};
