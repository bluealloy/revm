#[macro_use]
mod macros;

mod arithmetic;
mod bitwise;
mod control;
mod host;
mod host_env;
mod i256;
mod memory;
pub mod opcode;
mod stack;
mod system;

mod prelude {
    pub(super) use crate::primitives::{
        Bytes, Spec, SpecId, SpecId::*, B160, B256, KECCAK_EMPTY, U256,
    };
    pub(super) use crate::{gas, Host, InstructionResult, Interpreter};
    pub(super) use core::cmp::Ordering;
}

pub use opcode::{OpCode, OPCODE_JUMPMAP};
