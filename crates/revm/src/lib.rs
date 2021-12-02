#![allow(dead_code)]
//#![forbid(unsafe_code, unused_variables, unused_imports)]
//#![no_std] only blocker in auto_impl check: https://github.com/bluealloy/revm/issues/4

pub mod db;
mod evm;
mod evm_impl;
mod inspector;
mod instructions;
mod machine;
mod models;
mod spec;
mod subroutine;
mod util;

pub use evm_impl::{Host,EVMData};

pub type DummyStateDB = InMemoryDB;

pub use db::{Database, DatabaseCommit, InMemoryDB};
pub use evm::{new, EVM};
pub use inspector::{Inspector, NoOpInspector, OverrideSpec};
pub use instructions::{
    opcode::{self, OpCode, OPCODE_JUMPMAP},
    Return,
};
pub use machine::{Gas, Machine};
pub use models::*;
pub use spec::*;
pub use subroutine::{Account, SubRoutine};

extern crate alloc;

pub(crate) const USE_GAS: bool = !cfg!(feature = "no_gas_measuring");
