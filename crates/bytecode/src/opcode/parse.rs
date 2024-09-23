use super::OpCode;
use crate::opcode::NAME_TO_OPCODE;
use core::fmt;

/// An error indicating that an opcode is invalid.
#[derive(Debug, PartialEq, Eq)]
pub struct OpCodeError(());

impl fmt::Display for OpCodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid opcode")
    }
}

impl core::error::Error for OpCodeError {}

impl core::str::FromStr for OpCode {
    type Err = OpCodeError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(OpCodeError(()))
    }
}

impl OpCode {
    /// Parses an opcode from a string. This is the inverse of [`as_str`](Self::as_str).
    #[inline]
    pub fn parse(s: &str) -> Option<Self> {
        NAME_TO_OPCODE.get(s).copied()
    }
}
