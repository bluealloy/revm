//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

pub mod block;
pub mod cfg;
pub mod context;
pub mod context_wire;
pub mod journaled_state;
pub mod tx;

pub use block::BlockEnv;
pub use cfg::{Cfg, CfgEnv};
pub use context::*;
pub use context_wire::{ContextWire, ContextWiring};
pub use journaled_state::*;
pub use tx::TxEnv;
