#![allow(dead_code)]
//#![forbid(unsafe_code, unused_variables, unused_imports)]
#![no_std]

mod db;
mod error;
mod evm;
mod evm_impl;
mod inspector;
mod machine;
mod models;
mod opcode;
mod spec;
mod subroutine;
mod util;

use evm_impl::Handler;

pub use db::{Database, DatabaseCommit, DummyStateDB};
pub use error::*;
pub use evm::{new, EVM};
pub use inspector::{Inspector, NoOpInspector};
pub use machine::Machine;
pub use models::*;
pub use opcode::Control;
pub use spec::*;
pub use subroutine::Account;

/// libraries for no_std flag
//#[cfg(no_std)]
extern crate alloc;
pub use alloc::{
    borrow::{Borrow, Cow},
    collections::{btree_map::Entry, BTreeMap as Map},
    vec,
    vec::Vec,
};