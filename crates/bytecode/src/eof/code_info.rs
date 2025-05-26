use primitives::STACK_LIMIT;

use super::{
    decode_helpers::{consume_u16, consume_u8},
    EofDecodeError,
};
use std::vec::Vec;

/// Non returning function has a output `0x80`
const EOF_NON_RETURNING_FUNCTION: u8 = 0x80;

/// Types section that contains stack information for matching code section
#[derive(Debug, Clone, Default, Hash, PartialEq, Eq, Copy, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CodeInfo {
    /// `inputs` - 1 byte - `0x00-0x7F`
    ///
    /// Number of stack elements the code section consumes
    pub inputs: u8,
    /// `outputs` - 1 byte - `0x00-0x80`
    ///
    /// Number of stack elements the code section returns or 0x80 for non-returning functions
    pub outputs: u8,
    /// `max_stack_increase` - 2 bytes - `0x0000-0x03FF`
    ///
    /// Maximum number of elements that got added to the stack by this code section.
    pub max_stack_increase: u16,
}

impl CodeInfo {
    /// Returns new `CodeInfo` with the given inputs, outputs, and max_stack_increase.
    pub fn new(inputs: u8, outputs: u8, max_stack_increase: u16) -> Self {
        Self {
            inputs,
            outputs,
            max_stack_increase,
        }
    }

    /// Returns `true` if section is non-returning.
    pub fn is_non_returning(&self) -> bool {
        self.outputs == EOF_NON_RETURNING_FUNCTION
    }

    /// Calculates the difference between the number of input and output stack elements.
    #[inline]
    pub const fn io_diff(&self) -> i32 {
        self.outputs as i32 - self.inputs as i32
    }

    /// Encodes the section into the buffer.
    #[inline]
    pub fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.push(self.inputs);
        buffer.push(self.outputs);
        buffer.extend_from_slice(&self.max_stack_increase.to_be_bytes());
    }

    /// Decodes the section from the input.
    #[inline]
    pub fn decode(input: &[u8]) -> Result<(Self, &[u8]), EofDecodeError> {
        let (input, inputs) = consume_u8(input)?;
        let (input, outputs) = consume_u8(input)?;
        let (input, max_stack_increase) = consume_u16(input)?;
        let section = Self {
            inputs,
            outputs,
            max_stack_increase,
        };
        section.validate()?;
        Ok((section, input))
    }

    /// Validates the section.
    pub fn validate(&self) -> Result<(), EofDecodeError> {
        if self.inputs > 0x7f {
            return Err(EofDecodeError::InvalidCodeInfoInputValue { value: self.inputs });
        }

        if self.outputs > 0x80 {
            return Err(EofDecodeError::InvalidCodeInfoOutputValue {
                value: self.outputs,
            });
        }

        if self.max_stack_increase > 0x03FF {
            return Err(EofDecodeError::InvalidCodeInfoMaxIncrementValue {
                value: self.max_stack_increase,
            });
        }

        if self.inputs as usize + self.max_stack_increase as usize > STACK_LIMIT {
            return Err(EofDecodeError::InvalidCodeInfoStackOverflow {
                inputs: self.inputs,
                max_stack_increment: self.max_stack_increase,
            });
        }

        Ok(())
    }
}
