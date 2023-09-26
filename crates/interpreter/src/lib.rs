#![cfg_attr(not(feature = "std"), no_std)]
#![warn(unused_crate_dependencies)]

extern crate alloc;

#[macro_use]
mod macros;

pub mod gas;
mod host;
pub mod inner_models;
pub mod instruction_result;
pub mod instructions;
mod interpreter;

pub(crate) const USE_GAS: bool = !cfg!(feature = "no_gas_measuring");

// Reexport primary types.
pub use gas::Gas;
pub use host::{DummyHost, Host};
pub use inner_models::*;
pub use instruction_result::InstructionResult;
pub use instructions::{opcode, Instruction, OpCode, OPCODE_JUMPMAP};
pub use interpreter::{
    analysis, BytecodeLocked, Contract, Interpreter, Memory, Stack, CALL_STACK_LIMIT,
    MAX_CODE_SIZE, MAX_INITCODE_SIZE,
};

#[doc(inline)]
pub use revm_primitives as primitives;
