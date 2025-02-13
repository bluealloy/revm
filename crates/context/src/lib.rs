//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

pub mod block;
pub mod cfg;
pub mod context;
pub mod evm;
mod journal_init;
pub mod journaled_state;
pub mod tx;

pub use block::BlockEnv;
pub use cfg::{Cfg, CfgEnv};
pub use context::*;
pub use journal_init::JournalInit;
pub use journaled_state::*;
pub use tx::{AccessList, SignedAuthorization, TxEnv};
pub mod setters;
pub use evm::{Evm, EvmData};
