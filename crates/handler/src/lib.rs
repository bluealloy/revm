//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

// Use serde to avoid unused dependency warning when the serde feature is enabled
#[cfg(feature = "serde")]
use serde as _;

// Mainnet related handlers.

pub mod api;
pub mod evm;
pub mod execution;
mod frame;
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
pub use evm::{EvmTr, NewFrameTr};
pub use frame::{return_create, ContextTrDbError, EthFrameInner};
pub use handler::{EvmTrError, Handler};
pub use item_or_result::{ItemOrResult, NewFrameTrInitOrResult};
pub use mainnet_builder::{MainBuilder, MainContext, MainnetContext, MainnetEvm};
pub use mainnet_handler::MainnetHandler;
pub use precompile_provider::{EthPrecompiles, PrecompileProvider};
pub use system_call::{SystemCallCommitEvm, SystemCallEvm, SystemCallTx, SYSTEM_ADDRESS};
