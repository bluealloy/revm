//! # revm-interpreter
//!
//! REVM Interpreter.
#![warn(unreachable_pub, unused_crate_dependencies)]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

extern crate alloc;

#[macro_use]
mod macros;

pub mod gas;
mod host;
mod inner_models;
mod instruction_result;
pub mod instructions;
mod interpreter;

pub(crate) const USE_GAS: bool = !cfg!(feature = "no_gas_measuring");

// Reexport primary types.
pub use gas::Gas;
pub use host::{DummyHost, Host};
pub use inner_models::*;
pub use instruction_result::*;
pub use instructions::{opcode, Instruction, OpCode, OPCODE_JUMPMAP};
pub use interpreter::{
    analysis, next_multiple_of_32, BytecodeLocked, Contract, Interpreter, SharedMemory, Stack,
    MAX_CODE_SIZE, MAX_INITCODE_SIZE, STACK_LIMIT,
};
#[doc(hidden)]
pub use revm_primitives as primitives;
