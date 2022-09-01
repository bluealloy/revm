#![allow(dead_code)]
//#![no_std]

pub mod db;
mod evm;
mod evm_impl;
pub(crate) mod gas;
mod inspector;
mod instructions;
mod interpreter;
mod journaled_state;
mod models;
mod specification;

pub use evm_impl::{EVMData, Host};

pub type DummyStateDB = InMemoryDB;

pub use db::{Database, DatabaseCommit, InMemoryDB};
pub use evm::{evm_inner, new, EVM};
pub use gas::Gas;
pub use inspector::{Inspector, NoOpInspector};
pub use instructions::{
    opcode::{self, spec_opcode_gas, OpCode, OPCODE_JUMPMAP},
    Return,
};
pub use interpreter::{
    Bytecode, BytecodeLocked, BytecodeState, Contract, Interpreter, Memory, Stack,
};
pub use journaled_state::{Account, JournalEntry, JournaledState};
pub use models::*;
pub use specification::*;

extern crate alloc;

pub(crate) const USE_GAS: bool = !cfg!(feature = "no_gas_measuring");

// reexport `revm_precompiles`
pub mod precompiles {
    pub use revm_precompiles::*;
}
