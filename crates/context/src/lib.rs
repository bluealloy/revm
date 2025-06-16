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
pub mod frame;
pub mod frame_data;
pub mod journal;
pub mod local;
pub mod tx;

pub use block::BlockEnv;
pub use cfg::{Cfg, CfgEnv};
pub use context::*;
pub use evm::Evm;
pub use frame_data::{FrameData, FrameResult};
pub use journal::*;
pub use local::LocalContext;
pub use tx::TxEnv;
