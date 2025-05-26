//! Revm is a Rust EVM implementation.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

// reexport dependencies
pub use bytecode;
pub use context;
pub use context_interface;
pub use database;
pub use database_interface;
pub use handler;
pub use inspector;
pub use interpreter;
pub use precompile;
pub use primitives;
pub use state;

// Export items.

pub use context::journal::{Journal, JournalEntry};
pub use context::Context;
pub use database_interface::{Database, DatabaseCommit, DatabaseRef};
pub use handler::{
    ExecuteCommitEvm, ExecuteEvm, MainBuilder, MainContext, MainnetEvm, SystemCallCommitEvm,
    SystemCallEvm,
};
pub use inspector::{InspectCommitEvm, InspectEvm, Inspector};
