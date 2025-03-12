//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

pub use context_interface::*;

pub mod block;
pub mod cfg;
pub mod context;
pub mod evm;
pub mod journaled_state;
pub mod tx;

pub use block::BlockEnv;
pub use cfg::{Cfg, CfgEnv};
pub use context::*;
pub use evm::{Evm, EvmData};
pub use journaled_state::*;
pub use tx::TxEnv;
