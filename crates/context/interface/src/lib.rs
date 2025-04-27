//! Shared interfaces and traits for OP Stack (Optimism) extensions to revm.
//!
//! Defines the core traits and types for block, transaction, context, and configuration
//! used by Optimism-specific execution logic. These interfaces enable modular integration
//! of OP Stack features into revm-based EVMs.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

pub mod block;
pub mod cfg;
pub mod context;
pub mod journaled_state;
pub mod local;
pub mod result;
pub mod transaction;

pub use block::Block;
pub use cfg::{Cfg, CreateScheme, TransactTo};
pub use context::{ContextSetters, ContextTr};
pub use database_interface::{DBErrorMarker, Database};
pub use either;
pub use journaled_state::JournalTr;
pub use local::LocalContextTr;
pub use transaction::{Transaction, TransactionType};
