//! # revm-primitives
//!
//! EVM primitive types.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

mod bytecode;
mod constants;
pub mod db;
pub mod eip7702;
pub mod env;

#[cfg(any(feature = "c-kzg", feature = "kzg-rs"))]
pub mod kzg;
pub mod precompile;
pub mod result;
pub mod specification;
pub mod state;
pub mod utilities;
pub use alloy_eip2930::{AccessList, AccessListItem};
pub use alloy_primitives::{
    self, address, b256, bytes, fixed_bytes, hex, hex_literal,
    map::{self, hash_map, hash_set, HashMap, HashSet},
    ruint, uint, Address, Bytes, FixedBytes, Log, LogData, TxKind, B256, I256, U256,
};
pub use bitvec;
pub use bytecode::*;
pub use constants::*;
pub use eip7702::{
    Authorization, AuthorizationList, Eip7702Bytecode, Eip7702DecodeError, PrimitiveSignature,
    RecoveredAuthority, RecoveredAuthorization, SignedAuthorization, EIP7702_MAGIC,
    EIP7702_MAGIC_BYTES, EIP7702_MAGIC_HASH,
};
pub use env::*;

#[cfg(any(feature = "c-kzg", feature = "kzg-rs"))]
pub use kzg::{EnvKzgSettings, KzgSettings};
pub use precompile::*;
pub use result::*;
pub use specification::*;
pub use state::*;
pub use utilities::*;

#[cfg(all(feature = "c-kzg", feature = "kzg-rs"))]
// silence kzg-rs lint as c-kzg will be used as default if both are enabled.
use kzg_rs as _;
