//! Optimism-specific implementation for REVM (Rust Ethereum Virtual Machine).
//!
//! This module provides a complete implementation of Optimism's execution layer,
//! including support for all Optimism hardforks: Bedrock, Regolith, Canyon, Ecotone, etc.
//!
//! Key features:
//! - Support for Optimism's L1/L2 fee model and data gas calculations
//! - Deposit transactions and special transaction handling
//! - Hardfork-specific EVM modifications and precompiles
//! - L1 block information and fee scalar handling
//!
//! Optimism is an Ethereum L2 scaling solution that inherits security from Ethereum
//! while providing lower costs and higher throughput.
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
