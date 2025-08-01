//! # revm-primitives
//!
//! EVM primitive types.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

pub mod constants;
pub mod eip170;
pub mod eip3860;
pub mod eip4844;
pub mod eip7702;
pub mod eip7823;
pub mod eip7825;
pub mod eip7907;
pub mod eip7918;
pub mod hardfork;
mod once_lock;

pub use constants::*;
pub use once_lock::OnceLock;

// Reexport alloy primitives.

pub use alloy_primitives::map::{self, hash_map, hash_set, HashMap, HashSet};
pub use alloy_primitives::{
    self, address, b256, bytes, fixed_bytes, hex, hex_literal, keccak256, ruint, uint, Address,
    Bytes, FixedBytes, Log, LogData, TxKind, B256, I128, I256, U128, U256,
};

/// type alias for storage keys
pub type StorageKey = U256;
/// type alias for storage values
pub type StorageValue = U256;

/// Hints to the compiler that this is a cold path, i.e. unlikely to be taken.
#[cold]
#[inline(always)]
pub fn cold_path() {
    // TODO: remove `#[cold]` and call `std::hint::cold_path` once stable.
}
