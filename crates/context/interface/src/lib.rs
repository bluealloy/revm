//! Optimism-specific constants, types, and helpers for context interfaces.
//!
//! Extends standard interfaces with:
//! - Transaction: Deposit transaction type and validation
//! - Block: L1 block reference fields for fee calculation
//! - Result: Optimism-specific halt conditions for deposits
//! - Configuration: Optimism hardfork specification support
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
