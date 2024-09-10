//! # revm-primitives
//!
//! EVM primitive types.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

mod constants;
#[cfg(any(feature = "c-kzg", feature = "kzg-rs"))]
pub mod kzg;
pub mod specification;
pub use alloy_primitives::{
    self, address, b256, bytes, fixed_bytes, hex, hex_literal, keccak256, ruint, uint, Address,
    Bytes, FixedBytes, Log, LogData, TxKind, B256, I256, U256,
};
pub use bitvec;
pub use constants::*;

cfg_if::cfg_if! {
    if #[cfg(all(not(feature = "hashbrown"), feature = "std"))] {
        pub use std::collections::{hash_map, hash_set, HashMap, HashSet};
        use hashbrown as _;
    } else {
        pub use hashbrown::{hash_map, hash_set, HashMap, HashSet};
    }
}

#[cfg(any(feature = "c-kzg", feature = "kzg-rs"))]
pub use kzg::{EnvKzgSettings, KzgSettings};
pub use specification::*;

#[cfg(all(feature = "c-kzg", feature = "kzg-rs"))]
// silence kzg-rs lint as c-kzg will be used as default if both are enabled.
use kzg_rs as _;

/// The Keccak-256 hash of the empty string `""`.
pub const KECCAK_EMPTY: B256 =
    b256!("c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470");
