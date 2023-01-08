#![allow(dead_code)]
//#![no_std]

pub mod bits;
pub mod common;
pub mod gas;
mod host;
mod instructions;
mod interpreter;
pub mod models;
pub mod specification;

extern crate alloc;

pub(crate) const USE_GAS: bool = !cfg!(feature = "no_gas_measuring");

// Reexport primary types.
pub use bits::{B160, B256};
pub use bytes::Bytes;
pub use gas::Gas;
pub use host::{DummyHost, Host};
pub use instructions::{
    opcode::{self, spec_opcode_gas, OpCode, OPCODE_JUMPMAP},
    Return,
};
pub use interpreter::*;
pub use interpreter::{
    Bytecode, BytecodeLocked, BytecodeState, Contract, Interpreter, Memory, Stack,
};
pub use models::*;
pub use ruint;
pub use ruint::aliases::U256;
pub use specification::*;
