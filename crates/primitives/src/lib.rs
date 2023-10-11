//! # revm-primitives
//!
//! EVM primitive types.
#![warn(unreachable_pub, unused_crate_dependencies)]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

extern crate alloc;

mod bytecode;
mod constants;
pub mod db;
pub mod env;
#[cfg(feature = "c-kzg")]
pub mod kzg;
mod log;
pub mod precompile;
pub mod result;
pub mod specification;
pub mod state;
pub mod utilities;

pub use alloy_primitives::{
    self, address, b256, bytes, fixed_bytes, hex, hex_literal, ruint, uint, Address, Bytes,
    FixedBytes, B256, U256,
};
pub use bitvec;
pub use bytecode::*;
pub use constants::*;
pub use env::*;
pub use hashbrown::{hash_map, hash_set, HashMap, HashSet};
#[cfg(feature = "c-kzg")]
pub use kzg::{EnvKzgSettings, KzgSettings};
pub use log::*;
pub use precompile::*;
pub use result::*;
pub use specification::*;
pub use state::*;
pub use utilities::*;
