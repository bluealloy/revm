//! # revm-primitives
//!
//! EVM primitive types.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

pub mod block;
pub mod db;
pub mod eip7702;
pub mod env;

mod bytecode;
mod constants;
mod evm_wiring;
#[cfg(any(feature = "c-kzg", feature = "kzg-rs"))]
pub mod kzg;
pub mod precompile;
pub mod result;
pub mod specification;
pub mod state;
pub mod transaction;
pub mod utilities;
pub use alloy_eips::eip2930::{AccessList, AccessListItem};
pub use alloy_primitives::{
    self, address, b256, bytes, fixed_bytes, hex, hex_literal, ruint, uint, Address, Bytes,
    FixedBytes, Log, LogData, TxKind, B256, I256, U256,
};
pub use bitvec;
pub use bytecode::*;
pub use constants::*;
pub use eip7702::{
    Authorization, AuthorizationList, Eip7702Bytecode, Eip7702DecodeError, InvalidAuthorization,
    RecoveredAuthorization, Signature, SignedAuthorization, EIP7702_MAGIC, EIP7702_MAGIC_BYTES,
};
pub use env::*;
pub use evm_wiring::*;

cfg_if::cfg_if! {
    if #[cfg(all(not(feature = "hashbrown"), feature = "std"))] {
        pub use std::collections::{hash_map, hash_set, HashMap, HashSet};
        use hashbrown as _;
    } else {
        pub use hashbrown::{hash_map, hash_set, HashMap, HashSet};
    }
}

pub use block::Block;
#[cfg(any(feature = "c-kzg", feature = "kzg-rs"))]
pub use kzg::{EnvKzgSettings, KzgSettings};
pub use precompile::*;
pub use result::*;
pub use specification::*;
pub use state::*;
pub use transaction::Transaction;
pub use utilities::*;

#[cfg(all(feature = "c-kzg", feature = "kzg-rs"))]
// silence kzg-rs lint as c-kzg will be used as default if both are enabled.
use kzg_rs as _;
