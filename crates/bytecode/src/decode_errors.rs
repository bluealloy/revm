use crate::eip7702::Eip7702DecodeError;
use core::fmt::Debug;
use std::fmt;

/// Bytecode decode errors
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BytecodeDecodeError {
    /// EIP-7702 decode error
    Eip7702(Eip7702DecodeError),
}

impl From<Eip7702DecodeError> for BytecodeDecodeError {
    fn from(error: Eip7702DecodeError) -> Self {
        Self::Eip7702(error)
    }
}

impl core::error::Error for BytecodeDecodeError {}

impl fmt::Display for BytecodeDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Eip7702(e) => fmt::Display::fmt(e, f),
        }
    }
}
