//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

pub mod block;
pub mod evm_wiring;
pub mod default;
pub mod precompile;
pub mod transaction;


pub use block::Block;
pub use transaction::Transaction;