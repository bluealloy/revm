#![warn(unreachable_pub)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate alloc;

pub mod db;
mod evm;
mod evm_context;
mod evm_impl;
pub mod handler;
mod journaled_state;

mod gas;
pub mod mainnet;
mod types;

pub type DummyStateDB = InMemoryDB;
#[cfg(feature = "std")]
pub use db::{
    CacheState,
    DBBox,
    State,
    StateBuilder,
    StateDBBox,
    TransitionAccount,
    TransitionState,
};
pub use db::{Database, DatabaseCommit, DatabaseRef, InMemoryDB};
pub use evm::{evm_inner, new, EVM};
pub use evm_context::EVMData;
pub use evm_impl::{EVMImpl, Transact, CALL_STACK_LIMIT};
pub use handler::Handler;
pub use journaled_state::{JournalCheckpoint, JournalEntry, JournaledState};
// reexport `revm_primitives`
#[doc(inline)]
pub use revm_primitives as primitives;
