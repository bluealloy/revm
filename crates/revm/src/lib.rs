//! Revm is a Rust EVM implementation.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
#[cfg(not(feature = "std"))]
extern crate alloc as std;

// reexport dependencies
pub use bytecode;
pub use context;
pub use database_interface;
pub use interpreter;
pub use precompile;
pub use primitives;
pub use specification;
pub use state;
pub use transaction;
pub use wiring;

// Define modules.
mod builder;

mod evm;
mod evm_wiring;
pub mod handler;

// Export items.

pub use builder::EvmBuilder;
pub use context::frame::{CallFrame, CreateFrame, Frame, FrameData, FrameOrResult, FrameResult};
pub use context::journaled_state::{JournalCheckpoint, JournalEntry, JournaledState};
pub use context::{
    Context, ContextPrecompile, ContextPrecompiles, ContextStatefulPrecompile,
    ContextStatefulPrecompileArc, ContextStatefulPrecompileBox, ContextStatefulPrecompileMut,
    ContextWithEvmWiring, EvmContext, InnerEvmContext,
};
pub use database_interface::{Database, DatabaseCommit, DatabaseRef};
pub use evm::Evm;
pub use evm_wiring::EvmWiring;
pub use handler::{register::EvmHandler, Handler};
