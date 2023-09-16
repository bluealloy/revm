#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod bits;
pub mod bytecode;
pub mod constants;
pub mod db;
pub mod env;
#[cfg(feature = "std")]
pub mod kzg;
pub mod log;
pub mod precompile;
pub mod result;
pub mod specification;
pub mod state;
pub mod utilities;

pub use bits::B160;
pub use bits::B256;
pub use bitvec;
pub use bytecode::*;
pub use bytes;
pub use bytes::Bytes;
pub use constants::*;
pub use env::*;
pub use hashbrown::{hash_map, hash_set, HashMap, HashSet};
pub use hex;
pub use hex_literal;
#[cfg(feature = "std")]
pub use kzg::{EnvKzgSettings, KzgSettings};
pub use log::Log;
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
