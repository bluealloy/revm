//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

pub mod bn128;
pub mod evm;
pub mod fast_lz;
pub mod handler;
pub mod l1block;
pub mod result;
pub mod spec;
pub mod transaction;

pub use evm::L1BlockInfoGetter;
pub use l1block::{calculate_tx_l1_cost, BASE_FEE_RECIPIENT, L1_FEE_RECIPIENT};
pub use result::OptimismHaltReason;
pub use spec::*;
pub use transaction::{error::OpTransactionError, estimate_tx_compressed_size, OpTransaction};
