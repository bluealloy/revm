//! # revm-interpreter
//!
//! REVM Interpreter.
#![warn(rustdoc::all)]
#![warn(unreachable_pub, unused_crate_dependencies)]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

#[macro_use]
mod macros;

mod call_inputs;
mod call_outcome;
mod create_inputs;
mod create_outcome;
mod eof_create_inputs;
mod eof_create_outcome;
mod function_stack;
pub mod gas;
mod host;
mod instruction_result;
pub mod instructions;
mod interpreter;

// Reexport primary types.
pub use call_inputs::{CallContext, CallInputs, CallScheme, Transfer};
pub use call_outcome::CallOutcome;
pub use create_inputs::{CreateInputs, CreateScheme};
pub use create_outcome::CreateOutcome;
pub use eof_create_inputs::EOFCreateInput;
pub use eof_create_outcome::EOFCreateOutcome;
pub use function_stack::{FunctionReturnFrame, FunctionStack};
pub use gas::Gas;
pub use host::{DummyHost, Host, LoadAccountResult, SStoreResult, SelfDestructResult};
pub use instruction_result::*;
pub use instructions::{opcode, Instruction, OpCode, OPCODE_JUMPMAP};
pub use interpreter::{
    analysis, next_multiple_of_32, Contract, Interpreter, InterpreterAction, InterpreterResult,
    SharedMemory, Stack, EMPTY_SHARED_MEMORY, STACK_LIMIT,
};
pub use primitives::{MAX_CODE_SIZE, MAX_INITCODE_SIZE};

#[doc(hidden)]
pub use revm_primitives as primitives;
