//! Revm is a Rust EVM implementation.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
#[cfg(not(feature = "std"))]
extern crate alloc as std;

// Define modules.

mod builder;
mod context;

#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils;

pub mod db;
mod evm;
mod evm_wiring;
mod frame;
pub mod handler;
mod inspector;
mod journaled_state;

// Export items.

pub use builder::EvmBuilder;
pub use context::{
    Context, ContextPrecompile, ContextPrecompiles, ContextStatefulPrecompile,
    ContextStatefulPrecompileArc, ContextStatefulPrecompileBox, ContextStatefulPrecompileMut,
    ContextWithEvmWiring, EvmContext, InnerEvmContext,
};
pub use db::{
    CacheState, DBBox, State, StateBuilder, StateDBBox, TransitionAccount, TransitionState,
};
pub use db::{Database, DatabaseCommit, DatabaseRef, InMemoryDB};
pub use evm::{Evm, CALL_STACK_LIMIT};
pub use evm_wiring::EvmWiring;
pub use frame::{CallFrame, CreateFrame, Frame, FrameData, FrameOrResult, FrameResult};
pub use handler::{register::EvmHandler, Handler};
pub use inspector::{inspector_handle_register, inspectors, GetInspector, Inspector};
pub use journaled_state::{JournalCheckpoint, JournalEntry, JournaledState};
// Reexport libraries

#[doc(inline)]
pub use revm_interpreter as interpreter;
#[doc(inline)]
pub use revm_interpreter::primitives;
#[doc(inline)]
pub use revm_precompile as precompile;
