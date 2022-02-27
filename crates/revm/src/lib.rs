#![allow(dead_code)]
//#![no_std]

pub mod db;
mod evm;
mod evm_impl;
pub(crate) mod gas;
mod inspector;
mod instructions;
mod interpreter;
mod models;
mod specification;
mod subroutine;

pub use evm_impl::{EVMData, Host};

pub type DummyStateDB = InMemoryDB;

pub use db::{Database, DatabaseCommit, InMemoryDB};
pub use evm::{new, EVM};
pub use gas::Gas;
pub use inspector::{Inspector, NoOpInspector, OverrideSpec};
pub use instructions::{
    opcode::{self, spec_opcode_gas, OpCode, OPCODE_JUMPMAP},
    Return,
};
pub use interpreter::{Contract, Interpreter, Memory, Stack};
pub use models::*;
pub use specification::*;
pub use subroutine::{Account, Filth, SubRoutine};

extern crate alloc;

pub(crate) const USE_GAS: bool = !cfg!(feature = "no_gas_measuring");
