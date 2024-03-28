//! # revm-primitives
//!
//! EVM primitive types.
#![warn(rustdoc::all)]
#![warn(unreachable_pub, unused_crate_dependencies)]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

mod bytecode;
mod constants;
pub mod db;
pub mod env;
#[cfg(feature = "c-kzg")]
pub mod kzg;
pub mod precompile;
pub mod result;
pub mod specification;
pub mod state;
pub mod utilities;
pub use alloy_primitives::{
    self, address, b256, bytes, fixed_bytes, hex, hex_literal, ruint, uint, Address, Bytes,
    FixedBytes, Log, LogData, B256, I256, U256,
};
pub use bitvec;
pub use bytecode::*;
pub use constants::*;
pub use env::*;

cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
        pub use std::collections::{hash_map, hash_set, HashMap, HashSet};
        use hashbrown as _;
    } else {
        pub use hashbrown::{hash_map, hash_set, HashMap, HashSet};
    }
}

#[cfg(feature = "c-kzg")]
pub use kzg::{EnvKzgSettings, KzgSettings};
pub use precompile::*;
pub use result::*;
pub use specification::*;
pub use state::*;
pub use utilities::*;
