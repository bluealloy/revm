//! EIP-7702 bytecode constants and error types.

use core::fmt;
use primitives::{b256, hex, B256};

/// Hash of EF01 bytes that is used for EXTCODEHASH when called from legacy bytecode.
pub const EIP7702_MAGIC_HASH: B256 =
    b256!("0xeadcdba66a79ab5dce91622d1d75c8cff5cff0b96944c3bf1072cd08ce018329");

/// EIP-7702 Version Magic in u16 form.
pub const EIP7702_MAGIC: u16 = 0xEF01;

/// EIP-7702 magic number in array form.
pub const EIP7702_MAGIC_BYTES: &[u8] = &hex!("ef01");

/// EIP-7702 first version of bytecode.
pub const EIP7702_VERSION: u8 = 0;

/// EIP-7702 bytecode length: 2 (magic) + 1 (version) + 20 (address) = 23 bytes.
pub const EIP7702_BYTECODE_LEN: usize = 23;

/// EIP-7702 decode errors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Eip7702DecodeError {
    /// Invalid length of the raw bytecode.
    ///
    /// It should be 23 bytes.
    InvalidLength,
    /// Invalid magic number.
    ///
    /// All EIP-7702 bytecodes should start with the magic number 0xEF01.
    InvalidMagic,
    /// Unsupported version.
    ///
    /// Only supported version is version 0x00.
    UnsupportedVersion,
}

impl fmt::Display for Eip7702DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::InvalidLength => "Eip7702 is not 23 bytes long",
            Self::InvalidMagic => "Bytecode is not starting with 0xEF01",
            Self::UnsupportedVersion => "Unsupported Eip7702 version.",
        };
        f.write_str(s)
    }
}

impl core::error::Error for Eip7702DecodeError {}
