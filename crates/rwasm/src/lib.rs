#![warn(unreachable_pub)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate fluentbase_sdk;
#[macro_use]
extern crate alloc;

mod context;
pub mod db;
mod evm;
pub mod handler;
mod r#impl;
mod journal;

mod gas;
mod types;

pub type DummyStateDB = InMemoryDB;
pub use context::EVMData;
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
pub use evm::{evm_inner, new, RWASM};
pub use handler::Handler;
pub use r#impl::{EVMImpl, Transact, CALL_STACK_LIMIT};
// reexport `revm_primitives`
#[doc(inline)]
pub use revm_primitives as primitives;
