//! Crate that contains bytecode types and opcode constants.
//!
//! Legacy bytecode will always contain a jump table.
//!
//! While EIP-7702 bytecode must contains a Address.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

pub mod bytecode;
mod decode_errors;
/// EIP-7702 bytecode.
pub mod eip7702;
/// Iterator for the bytecode.
mod iter;
/// Legacy bytecode.
pub mod legacy;
pub mod opcode;
/// Metadata account
pub mod ownable_account;
pub mod utils;

/// Re-export of bitvec crate, used to store legacy bytecode jump table.
pub use bitvec;
pub use bytecode::Bytecode;
pub use decode_errors::BytecodeDecodeError;
pub use iter::BytecodeIterator;
pub use legacy::{JumpTable, LegacyAnalyzedBytecode, LegacyRawBytecode};
pub use opcode::OpCode;
use primitives::Bytes;

/// Rwasm magic number in array form.
pub static RWASM_MAGIC_BYTES: Bytes = primitives::bytes!("ef52");
/// Wasm magic number in array form.
pub static WASM_MAGIC_BYTES: Bytes = primitives::bytes!("0061736d");
/// SVM magic number in array form.
pub static SVM_ELF_MAGIC_BYTES: Bytes = primitives::bytes!("7f454c46");
