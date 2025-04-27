//! OP Stack (Optimism) integration for revm: provides types, constants, and helpers for L2 execution.
//!
//! This crate extends revm with Optimism-specific logic, including:
//! - L1 fee calculation and storage slots
//! - OP Stack transaction types and validation
//! - L1 block info and context
//! - OP Stack-specific precompiles and handler logic
//!
//! Use this crate when building EVM execution environments or tools for Optimism or compatible L2s.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

pub mod api;
pub mod constants;
pub mod evm;
pub mod fast_lz;
pub mod handler;
pub mod l1block;
pub mod precompiles;
pub mod result;
pub mod spec;
pub mod transaction;

pub use api::{
    builder::OpBuilder,
    default_ctx::{DefaultOp, OpContext},
};
pub use evm::OpEvm;
pub use l1block::L1BlockInfo;
pub use result::OpHaltReason;
pub use spec::*;
pub use transaction::{error::OpTransactionError, estimate_tx_compressed_size, OpTransaction};
