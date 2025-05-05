//! Crate that contains bytecode types and opcode constants.
//!
//! EOF bytecode contains its verification logic and only valid EOF bytecode can be created.
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
pub mod eof;
/// Iterator for the bytecode.
mod iter;
/// Legacy bytecode.
pub mod legacy;
pub mod opcode;
pub mod utils;

/// Re-export of bitvec crate, used to store legacy bytecode jump table.
pub use bitvec;
pub use bytecode::Bytecode;
pub use decode_errors::BytecodeDecodeError;
pub use eof::{
    verification::{
        validate_eof, validate_eof_code, validate_eof_codes, validate_eof_inner, validate_raw_eof,
        validate_raw_eof_inner, CodeType, EofValidationError,
    },
    Eof, EOF_MAGIC, EOF_MAGIC_BYTES, EOF_MAGIC_HASH,
};
pub use iter::BytecodeIterator;
pub use legacy::{JumpTable, LegacyAnalyzedBytecode, LegacyRawBytecode};
pub use opcode::OpCode;
