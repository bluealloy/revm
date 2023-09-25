#![warn(missing_debug_implementations, unreachable_pub)]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

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
