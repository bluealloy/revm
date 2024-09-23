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

mod function_stack;
pub mod gas;
mod host;
mod instruction_result;
pub mod instructions;
pub mod interpreter;
pub mod interpreter_action;
pub mod table;

// Reexport primary types.
pub use function_stack::{FunctionReturnFrame, FunctionStack};
pub use gas::Gas;
pub use host::{
    AccountLoad, DummyHost, Eip7702CodeLoad, Host, SStoreResult, SelfDestructResult, StateLoad,
};
pub use instruction_result::*;
pub use interpreter::{
    num_words, Contract, Interpreter, InterpreterResult, SharedMemory, Stack, EMPTY_SHARED_MEMORY,
    STACK_LIMIT,
};
pub use interpreter_action::{
    CallInputs, CallOutcome, CallScheme, CallValue, CreateInputs, CreateOutcome, EOFCreateInputs,
    EOFCreateKind, InterpreterAction,
};
pub use primitives::{MAX_CODE_SIZE, MAX_INITCODE_SIZE};
pub use table::Instruction;
pub use wiring::default::CreateScheme;
