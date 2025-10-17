//! EVM execution context.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

// database is used in dev-dependencies for tests and as optional dependency
#[cfg(any(test, feature = "database"))]
use database as _;

pub use context_interface::*;

pub mod block;
pub mod cfg;
pub mod context;
pub mod evm;
pub mod journal;
pub mod local;
pub mod tx;

pub use block::BlockEnv;
pub use cfg::{Cfg, CfgEnv};
pub use context::*;
pub use evm::Evm;
pub use journal::*;
pub use local::LocalContext;
pub use tx::TxEnv;
