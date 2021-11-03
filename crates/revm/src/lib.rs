#![allow(dead_code)]
//#![forbid(unsafe_code, unused_variables, unused_imports)]
//#![no_std] only blocker in auto_impl check: https://github.com/bluealloy/revm/issues/4

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

extern crate alloc;

pub(crate) const USE_GAS: bool = !cfg!(feature = "no_gas_measuring");
