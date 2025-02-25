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
mod instruction_result;
pub mod instructions;
pub mod interpreter;
pub mod interpreter_action;
pub mod interpreter_types;
pub mod table;

// Reexport primary types.
pub use context_interface::{
    context::{SStoreResult, SelfDestructResult, StateLoad},
    ContextTr as Host, CreateScheme,
};
pub use gas::{Gas, InitialAndFloorGas};
pub use instruction_result::*;
pub use interpreter::{
    num_words, InputsImpl, Interpreter, InterpreterResult, MemoryGetter, SharedMemory, Stack,
    EMPTY_SHARED_MEMORY, STACK_LIMIT,
};
pub use interpreter_action::{
    CallInputs, CallOutcome, CallScheme, CallValue, CreateInputs, CreateOutcome, EOFCreateInputs,
    EOFCreateKind, FrameInput, InterpreterAction,
};
pub use interpreter_types::InterpreterTypes;
pub use specification::constants::{MAX_CODE_SIZE, MAX_INITCODE_SIZE};
pub use table::Instruction;
