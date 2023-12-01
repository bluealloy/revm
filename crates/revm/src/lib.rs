#![warn(unreachable_pub)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(all(feature = "with-serde", not(feature = "serde")))]
compile_error!("`with-serde` feature has been renamed to `serde`.");

#[macro_use]
extern crate alloc;

// Define modules.

mod context;
pub mod db;
mod evm;
mod builder;
mod frame;
pub mod handler;
mod inspector;
mod journaled_state;
#[cfg(feature = "optimism")]
pub mod optimism;

// Export items.

pub use context::{Context, EvmContext};
#[cfg(feature = "std")]
pub use db::{
    CacheState, DBBox, State, StateBuilder, StateDBBox, TransitionAccount, TransitionState,
};
pub use db::{Database, DatabaseCommit, DatabaseRef, DummyStateDB, InMemoryDB};
pub use evm::{Evm, CALL_STACK_LIMIT};
pub use builder::EvmBuilder;
pub use frame::{CallStackFrame, FrameOrResult};
pub use handler::Handler;
pub use inspector::{inspector_instruction, inspectors, Inspector};
pub use journaled_state::{JournalCheckpoint, JournalEntry, JournaledState};
// export Optimism types, helpers, and constants
#[cfg(feature = "optimism")]
pub use optimism::{L1BlockInfo, BASE_FEE_RECIPIENT, L1_BLOCK_CONTRACT, L1_FEE_RECIPIENT};

// Reexport libraries

#[doc(inline)]
pub use revm_interpreter as interpreter;
#[doc(inline)]
pub use revm_interpreter::primitives;
#[doc(inline)]
pub use revm_precompile as precompile;
