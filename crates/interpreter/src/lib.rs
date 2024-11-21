//! # revm-interpreter
//!
//! REVM Interpreter.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

#[macro_use]
mod macros;

// silence lint
#[cfg(test)]
use serde_json as _;

#[cfg(test)]
use walkdir as _;

pub mod gas;
mod host;
mod instruction_result;
pub mod instructions;
pub mod interpreter;
pub mod interpreter_action;
pub mod interpreter_wiring;
pub mod table;

// Reexport primary types.
pub use context_interface::CreateScheme;
pub use gas::Gas;
pub use host::{DummyHost, Host, SStoreResult, SelfDestructResult, StateLoad};
pub use instruction_result::*;
pub use interpreter::{
    num_words, InputsImpl, InterpreterResult, MemoryGetter, NewInterpreter, SharedMemory, Stack,
    EMPTY_SHARED_MEMORY, STACK_LIMIT,
};
pub use interpreter_action::{
    CallInputs, CallOutcome, CallScheme, CallValue, CreateInputs, CreateOutcome, EOFCreateInputs,
    EOFCreateKind, InterpreterAction, NewFrameAction,
};
pub use interpreter_wiring::InterpreterWire;
pub use specification::constants::{MAX_CODE_SIZE, MAX_INITCODE_SIZE};
pub use table::Instruction;
