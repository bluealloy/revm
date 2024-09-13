//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

pub mod block;
pub mod default;
pub mod evm_wiring;
pub mod precompile;
pub mod result;
pub mod transaction;

pub use block::Block;
pub use evm_wiring::{DefaultEthereumWiring, EthereumWiring, EvmWiring};
pub use transaction::{Transaction, TransactionValidation};
