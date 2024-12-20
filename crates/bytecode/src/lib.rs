//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

pub mod bytecode;
pub mod decode_errors;
pub mod eip7702;
pub mod eof;
pub mod legacy;
pub mod opcode;
pub mod utils;

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
pub use legacy::{JumpTable, LegacyAnalyzedBytecode, LegacyRawBytecode};
