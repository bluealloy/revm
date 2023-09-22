//! # revm-primitives
//!
//! EVM primitive types.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

mod bits;
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

pub use bits::*;
pub use bitvec;
pub use bytecode::*;
pub use bytes;
pub use bytes::Bytes;
pub use constants::*;
pub use env::*;
pub use hashbrown::{hash_map, hash_set, HashMap, HashSet};
pub use hex;
pub use hex_literal;
#[cfg(feature = "c-kzg")]
pub use kzg::{EnvKzgSettings, KzgSettings};
pub use log::*;
pub use precompile::*;
pub use result::*;
pub use ruint;
pub use ruint::aliases::U256;
pub use ruint::uint;
pub use specification::*;
pub use state::*;
pub use utilities::*;

/// Address type is last 20 bytes of hash of ethereum account
pub type Address = B160;
/// Hash, in Ethereum usually keccak256.
pub type Hash = B256;
