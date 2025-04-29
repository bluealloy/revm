//! # revm-interpreter
//!
//! Interpreter is part of the project that executes EVM instructions.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

#[macro_use]
mod macros;

pub mod gas;
pub mod host;
mod instruction_result;
pub mod instructions;
pub mod interpreter;
pub mod interpreter_action;
pub mod interpreter_types;

// Reexport primary types.
pub use context_interface::{
    context::{SStoreResult, SelfDestructResult, StateLoad},
    CreateScheme,
};
pub use gas::{Gas, InitialAndFloorGas};
pub use host::Host;
pub use instruction_result::*;
pub use instructions::{instruction_table, Instruction, InstructionTable};
pub use interpreter::{
    num_words, InputsImpl, Interpreter, InterpreterResult, SharedMemory, Stack, STACK_LIMIT,
};
pub use interpreter_action::{
    CallInput, CallInputs, CallOutcome, CallScheme, CallValue, CreateInputs, CreateOutcome,
    EOFCreateInputs, EOFCreateKind, FrameInput, InterpreterAction,
};
pub use interpreter_types::InterpreterTypes;
pub use primitives::{constants::MAX_INITCODE_SIZE, eip170::MAX_CODE_SIZE};
