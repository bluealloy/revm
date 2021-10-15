#![allow(dead_code)]
//#![forbid(unsafe_code, unused_variables, unused_imports)]
//#![cfg_attr(not(feature = "std"), no_std)]

mod error;
mod evm;
mod evm_impl;
mod machine;
mod models;
mod opcode;
mod spec;
mod subroutine;
mod util;
mod db;
mod precompiles;
mod inspector;

use evm_impl::Handler;

extern crate alloc;

pub use db::{Database, StateDB};
pub use error::*;
pub use evm::{EVM,new,new_inspect};
pub use inspector::{Inspector,NoOpInspector};
pub use machine::Machine;
pub use opcode::Control;
pub use models::*;
pub use spec::*;
pub use subroutine::Account;

/// libraries for no_sdt flag
#[cfg(no_sdt)]
pub mod collection {
    pub use alloc::collections::{btree_map::Entry, BTreeMap as Map};
    pub use alloc::vec;
    pub use alloc::vec::Vec;
    pub use alloc::borrow::{Borrow,Cow};
}

#[cfg(not(no_sdt))]
pub mod collection {
    pub use std::collections::{hash_map::Entry, HashMap as Map};
    pub use std::vec;
    pub use std::vec::Vec;
    pub use std::borrow::{Cow,Cow::Borrowed};
}
