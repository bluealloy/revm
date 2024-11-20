//! Revm is a Rust EVM implementation.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
#[cfg(not(feature = "std"))]
extern crate alloc as std;

// reexport dependencies
pub use bytecode;
pub use context;
pub use context_interface;
pub use database_interface;
pub use interpreter;
pub use precompile;
pub use primitives;
pub use specification;
pub use state;

// Modules.

mod evm;
pub mod handler;
pub mod mainnet;

// Export items.

pub use context::journaled_state::{JournalEntry, JournaledState};
pub use context::Context;
pub use database_interface::{Database, DatabaseCommit, DatabaseRef};
pub use evm::{Error, EthContext, Evm, MainEvm};
