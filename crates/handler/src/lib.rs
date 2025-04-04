//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

// Mainnet related handlers.

pub mod api;
pub mod evm;
pub mod execution;
mod frame;
mod frame_data;
pub mod handler;
pub mod instructions;
mod item_or_result;
mod mainnet_builder;
mod mainnet_handler;
pub mod post_execution;
pub mod pre_execution;
mod precompile_provider;
pub mod system_call;
pub mod validation;

// Public exports
pub use api::{ExecuteCommitEvm, ExecuteEvm};
pub use evm::EvmTr;
pub use frame::{return_create, return_eofcreate, ContextTrDbError, EthFrame, Frame};
pub use frame_data::{FrameData, FrameResult};
pub use handler::{EvmTrError, Handler};
pub use item_or_result::{FrameInitOrResult, FrameOrResult, ItemOrResult};
pub use mainnet_builder::{MainBuilder, MainContext, MainnetContext, MainnetEvm};
pub use mainnet_handler::MainnetHandler;
pub use precompile_provider::{EthPrecompiles, PrecompileProvider};
pub use system_call::{SystemCallCommitEvm, SystemCallEvm, SystemCallTx, SYSTEM_ADDRESS};
