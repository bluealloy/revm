#![allow(dead_code)]
//#![no_std]

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

pub use evm_impl::{EVMData, Host};

pub type DummyStateDB = InMemoryDB;

pub use db::{Database, DatabaseCommit, InMemoryDB};
pub use evm::{new, EVM};
pub use inspector::{Inspector, NoOpInspector, OverrideSpec};
pub use instructions::{
    opcode::{self, spec_opcode_gas, OpCode, OPCODE_JUMPMAP},
    Return,
};
pub use machine::{Gas, Machine};
pub use models::*;
pub use spec::*;
pub use subroutine::{Account, SubRoutine};

extern crate alloc;

pub(crate) const USE_GAS: bool = !cfg!(feature = "no_gas_measuring");
