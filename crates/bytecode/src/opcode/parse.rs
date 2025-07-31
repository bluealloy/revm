//! Parsing opcodes from strings.
//!
//! This module provides a function to parse opcodes from strings.
//! It is a utility function that needs to be enabled with `parse` feature.

use super::OpCode;
use crate::opcode::NAME_TO_OPCODE;
use core::fmt;

/// An error indicating that an opcode is invalid
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
    /// Parses an opcode from a string.
    ///
    /// This is the inverse of [`as_str`](Self::as_str).
    #[inline]
    pub fn parse(s: &str) -> Option<Self> {
        NAME_TO_OPCODE.get(s).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::str::FromStr;

    #[test]
    fn test_opcode_error_display() {
        let err = OpCodeError(());
        assert_eq!(err.to_string(), "invalid opcode");
    }

    #[test]
    fn test_opcode_error_debug() {
        let err = OpCodeError(());
        let debug_str = format!("{:?}", err);
        assert_eq!(debug_str, "OpCodeError(())");
    }

    #[test]
    fn test_opcode_error_is_error() {
        let err = OpCodeError(());
        // This will fail to compile if Error trait is not implemented
        let _: &dyn core::error::Error = &err;
    }

    #[test]
    fn test_parse_valid_opcodes() {
        // Test some common opcodes
        assert_eq!(OpCode::parse("STOP"), Some(OpCode::STOP));
        assert_eq!(OpCode::parse("ADD"), Some(OpCode::ADD));
        assert_eq!(OpCode::parse("MUL"), Some(OpCode::MUL));
        assert_eq!(OpCode::parse("PUSH1"), Some(OpCode::PUSH1));
        assert_eq!(OpCode::parse("PUSH32"), Some(OpCode::PUSH32));
        assert_eq!(OpCode::parse("DUP1"), Some(OpCode::DUP1));
        assert_eq!(OpCode::parse("SWAP1"), Some(OpCode::SWAP1));
        assert_eq!(OpCode::parse("RETURN"), Some(OpCode::RETURN));
        assert_eq!(OpCode::parse("REVERT"), Some(OpCode::REVERT));
        assert_eq!(OpCode::parse("INVALID"), Some(OpCode::INVALID));
    }

    #[test]
    fn test_parse_invalid_opcodes() {
        assert_eq!(OpCode::parse("INVALID_OPCODE"), None);
        assert_eq!(OpCode::parse(""), None);
        assert_eq!(OpCode::parse("stop"), None); // Case sensitive
        assert_eq!(OpCode::parse("ADD "), None); // With space
        assert_eq!(OpCode::parse(" ADD"), None); // With space
        assert_eq!(OpCode::parse("PUSH"), None); // Incomplete
        assert_eq!(OpCode::parse("PUSH33"), None); // Out of range
    }

    #[test]
    fn test_from_str_valid() {
        assert_eq!(OpCode::from_str("STOP"), Ok(OpCode::STOP));
        assert_eq!(OpCode::from_str("ADD"), Ok(OpCode::ADD));
        assert_eq!(OpCode::from_str("PUSH1"), Ok(OpCode::PUSH1));
    }

    #[test]
    fn test_from_str_invalid() {
        assert_eq!(OpCode::from_str("INVALID_OPCODE"), Err(OpCodeError(())));
        assert_eq!(OpCode::from_str(""), Err(OpCodeError(())));
        assert_eq!(OpCode::from_str("stop"), Err(OpCodeError(())));
    }

    #[test]
    fn test_parse_inverse_of_as_str() {
        // Test that parse is the inverse of as_str for all valid opcodes
        for byte in 0..=255u8 {
            if let Some(opcode) = OpCode::new(byte) {
                let name = opcode.as_str();
                // Only test opcodes that have a proper name (not UNKNOWN)
                if !name.starts_with("UNKNOWN") {
                    assert_eq!(
                        OpCode::parse(name),
                        Some(opcode),
                        "Failed to parse {} back to opcode 0x{:02x}",
                        name,
                        byte
                    );
                }
            }
        }
    }

    #[test]
    fn test_all_named_opcodes_parseable() {
        // Ensure all opcodes in NAME_TO_OPCODE can be parsed
        for (name, &opcode) in NAME_TO_OPCODE.entries() {
            assert_eq!(
                OpCode::parse(name),
                Some(opcode),
                "Failed to parse {} from NAME_TO_OPCODE",
                name
            );
            assert_eq!(
                OpCode::from_str(name),
                Ok(opcode),
                "Failed to parse {} via FromStr",
                name
            );
        }
    }

    #[test]
    fn test_opcode_error_equality() {
        let err1 = OpCodeError(());
        let err2 = OpCodeError(());
        assert_eq!(err1, err2);
    }
}
