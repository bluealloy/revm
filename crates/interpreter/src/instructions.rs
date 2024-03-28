//! EVM opcode implementations.

#[macro_use]
pub mod macros;
pub mod arithmetic;
pub mod bitwise;
pub mod contract;
pub mod control;
pub mod data;
pub mod host;
pub mod host_env;
pub mod i256;
pub mod memory;
pub mod stack;
pub mod system;
pub mod utility;
