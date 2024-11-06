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

// silence kzg-rs lint as c-kzg will be used as default if both are enabled.

#[cfg(all(feature = "c-kzg", feature = "kzg-rs"))]
use kzg_rs as _;
#[cfg(all(feature = "c-kzg", feature = "kzg-rs"))]
use once_cell as _;
