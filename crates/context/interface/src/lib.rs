//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

pub mod block;
pub mod cfg;
pub mod context;
pub mod journaled_state;
pub mod result;
pub mod transaction;

pub use block::Block;
pub use cfg::{Cfg, CreateScheme, TransactTo};
pub use context::ContextTr;
pub use database_interface::{DBErrorMarker, Database};
pub use journaled_state::Journal;
pub use transaction::{Transaction, TransactionType};
