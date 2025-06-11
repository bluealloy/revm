//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

mod api;
mod evm;
mod executor;
mod frame;
mod result;
mod spec;
mod syscall;
mod types;

pub use api::*;
pub use evm::RwasmEvm;
pub use result::OpHaltReason;
pub use spec::*;
