//! Revm is a Rust EVM implementation.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

// reexport dependencies
#[doc(inline)]
pub use bytecode;
#[doc(inline)]
pub use context;
#[doc(inline)]
pub use context_interface;
#[doc(inline)]
pub use database;
#[doc(inline)]
pub use database_interface;
#[doc(inline)]
pub use handler;
#[doc(inline)]
pub use inspector;
#[doc(inline)]
pub use interpreter;
#[doc(inline)]
pub use precompile;
#[doc(inline)]
pub use primitives;
#[doc(inline)]
pub use state;

#[cfg(feature = "test-types")]
#[doc(inline)]
pub use statetest_types;

// Export items.

pub use context::{
    journal::{Journal, JournalEntry},
    Context,
};
#[cfg(feature = "asyncdb")]
pub use database_interface::{AsyncDb, AsyncError, AsyncResult, DatabaseAsync, WrapDatabaseAsync};
pub use database_interface::{Database, DatabaseCommit, DatabaseRef, NoopHook, OnStateHook};
pub use handler::{
    ExecuteCommitEvm, ExecuteEvm, MainBuilder, MainContext, MainnetEvm, SystemCallCommitEvm,
    SystemCallEvm,
};
#[cfg(feature = "asyncdb")]
pub use handler::{ExecuteEvmAsync, SystemCallEvmAsync};
pub use inspector::{InspectCommitEvm, InspectEvm, InspectSystemCallEvm, Inspector};
pub use precompile::install_crypto;
