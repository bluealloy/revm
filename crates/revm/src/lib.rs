#![cfg_attr(not(feature = "std"), no_std)]
#![warn(unreachable_pub)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

#[macro_use]
extern crate alloc;

pub mod db;
mod evm;
mod evm_impl;
pub mod handler;
mod inspector;
mod journaled_state;

#[cfg(feature = "optimism")]
pub mod optimism;

#[cfg(all(feature = "with-serde", not(feature = "serde")))]
compile_error!("`with-serde` feature has been renamed to `serde`.");

pub(crate) const USE_GAS: bool = !cfg!(feature = "no_gas_measuring");
pub type DummyStateDB = InMemoryDB;

#[cfg(feature = "std")]
pub use db::{
    CacheState, DBBox, State, StateBuilder, StateDBBox, TransitionAccount, TransitionState,
};

pub use db::{Database, DatabaseCommit, DatabaseRef, InMemoryDB};
pub use evm::{evm_inner, new, EVM};
pub use evm_impl::{EVMData, EVMImpl, Transact, CALL_STACK_LIMIT};
pub use journaled_state::{is_precompile, JournalCheckpoint, JournalEntry, JournaledState};

// reexport `revm_precompiles`
#[doc(inline)]
pub use revm_precompile as precompile;

// reexport `revm_interpreter`
#[doc(inline)]
pub use revm_interpreter as interpreter;

// reexport `revm_primitives`
#[doc(inline)]
pub use revm_interpreter::primitives;

// reexport inspector implementations
pub use inspector::inspectors;
pub use inspector::Inspector;

// export Optimism types, helpers, and constants
#[cfg(feature = "optimism")]
pub use optimism::{L1BlockInfo, BASE_FEE_RECIPIENT, L1_BLOCK_CONTRACT, L1_FEE_RECIPIENT};

pub use handler::Handler;
