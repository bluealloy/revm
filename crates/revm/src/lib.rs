//! Revm is a Rust EVM implementation.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(unused_extern_crates)]
#[macro_use]
#[cfg(not(feature = "std"))]
extern crate alloc as std;

#[cfg(feature = "std")]
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
#[cfg(feature = "revm-rwasm")]
mod journal_db_wrapper;
mod journaled_state;
#[cfg(feature = "optimism")]
pub mod optimism;

// Export items.

pub use builder::EvmBuilder;
pub use context::{
    Context,
    ContextPrecompile,
    ContextPrecompiles,
    ContextStatefulPrecompile,
    ContextStatefulPrecompileArc,
    ContextStatefulPrecompileBox,
    ContextStatefulPrecompileMut,
    ContextWithHandlerCfg,
    EvmContext,
    InnerEvmContext,
};
pub use db::{
    CacheState,
    DBBox,
    Database,
    DatabaseCommit,
    DatabaseRef,
    InMemoryDB,
    State,
    StateBuilder,
    StateDBBox,
    TransitionAccount,
    TransitionState,
};
pub use evm::{Evm, CALL_STACK_LIMIT};
pub use frame::{CallFrame, CreateFrame, Frame, FrameData, FrameOrResult, FrameResult};
pub use handler::Handler;
pub use inspector::{
    inspector_handle_register,
    inspector_instruction,
    inspectors,
    GetInspector,
    Inspector,
};
pub use journaled_state::{JournalCheckpoint, JournalEntry, JournaledState};
// export Optimism types, helpers, and constants
#[cfg(feature = "optimism")]
pub use optimism::{L1BlockInfo, BASE_FEE_RECIPIENT, L1_BLOCK_CONTRACT, L1_FEE_RECIPIENT};

// Reexport libraries
// #[cfg(feature = "revm-rwasm")]
extern crate core;
pub extern crate revm_interpreter_fluent as revm_interpreter;

#[doc(inline)]
pub use revm_interpreter as interpreter;
#[doc(inline)]
pub use revm_interpreter::primitives;
#[doc(inline)]
pub use revm_precompile as precompile;
