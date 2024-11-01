//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

pub mod context;
pub mod getters;
pub mod journaled_state;

pub use context::*;
pub use getters::*;
pub use journaled_state::*;
