#[macro_use]
pub mod macros;

pub mod arithmetic;
pub mod bitwise;
pub mod control;
pub mod host;
pub mod host_env;
pub mod i256;
pub mod memory;
pub mod opcode;
pub mod stack;
pub mod system;

pub use opcode::{Instruction, OpCode, OPCODE_JUMPMAP};
