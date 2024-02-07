#![warn(rustdoc::all, unreachable_pub)]
#![allow(rustdoc::bare_urls)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate alloc;

// Define modules.

mod builder;
mod context;

#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils;

pub mod db;
mod evm;
mod frame;
pub mod handler;
mod inspector;
mod journaled_state;
#[cfg(feature = "optimism")]
pub mod optimism;

// Export items.

pub use builder::EvmBuilder;
pub use context::{Context, EvmContext};
#[cfg(feature = "std")]
pub use db::{
    CacheState, DBBox, State, StateBuilder, StateDBBox, TransitionAccount, TransitionState,
};
pub use db::{Database, DatabaseCommit, DatabaseRef, InMemoryDB};
pub use evm::{Evm, CALL_STACK_LIMIT};
pub use frame::{CallFrame, CreateFrame, Frame, FrameData, FrameOrResult, FrameResult};
pub use handler::Handler;
pub use inspector::{
    inspector_handle_register, inspector_instruction, inspectors, GetInspector, Inspector,
};
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
