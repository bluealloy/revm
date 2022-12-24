#![allow(dead_code)]
//#![no_std]
pub mod db;
mod evm;
mod evm_impl;
mod inspector;
mod journaled_state;

pub(crate) const USE_GAS: bool = !cfg!(feature = "no_gas_measuring");
pub type DummyStateDB = InMemoryDB;

pub use db::{Database, DatabaseCommit, InMemoryDB};
pub use evm::{evm_inner, new, EVM};
pub use evm_impl::EVMData;
pub use journaled_state::{JournalEntry, JournaledState};
pub use revm_interpreter::*;

extern crate alloc;

/// reexport `revm_precompiles`
pub mod precompiles {
    pub use revm_precompiles::*;
}
// reexport `revm_interpreter`
pub mod interpreter {
    pub use revm_interpreter::*;
}

/// Reexport Inspector implementations
pub use inspector::inspectors;
pub use inspector::Inspector;
