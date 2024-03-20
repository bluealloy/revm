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

mod call_outcome;
mod create_outcome;
pub mod gas;
mod host;
mod inner_models;
mod instruction_result;
pub mod instructions;
pub mod interpreter;

// Reexport primary types.
pub use call_outcome::CallOutcome;
pub use create_outcome::CreateOutcome;
pub use gas::Gas;
pub use host::{DummyHost, Host, SStoreResult};
pub use inner_models::*;
pub use instruction_result::*;
pub use instructions::{opcode, Instruction, OpCode, OPCODE_JUMPMAP};
pub use interpreter::{
    analysis, next_multiple_of_32, BytecodeLocked, Contract, Interpreter, InterpreterAction,
    InterpreterResult, SharedMemory, Stack, EMPTY_SHARED_MEMORY, STACK_LIMIT,
};
pub use primitives::{MAX_CODE_SIZE, MAX_INITCODE_SIZE};

#[doc(hidden)]
pub use revm_primitives as primitives;
