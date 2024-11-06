//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

pub mod block;
pub mod cfg;
pub mod journaled_state;
pub mod precompile;
pub mod result;

pub use block::Block;
pub use cfg::{Cfg, CfgEnv, CreateScheme, TransactTo};
pub use transaction::{Transaction, TransactionType};
