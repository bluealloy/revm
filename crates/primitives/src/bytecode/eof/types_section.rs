use super::{
    decode_helpers::{consume_u16, consume_u8},
    EofDecodeError,
};
use std::vec::Vec;

/// Types section that contains stack information for matching code section.
#[derive(Debug, Clone, Default, Hash, PartialEq, Eq, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypesSection {
    /// inputs - 1 byte - `0x00-0x7F`
    /// number of stack elements the code section consumes
    pub inputs: u8,
    /// outputs - 1 byte - `0x00-0x80`
    /// number of stack elements the code section returns or 0x80 for non-returning functions
    pub outputs: u8,
    /// max_stack_height - 2 bytes - `0x0000-0x03FF`
    /// maximum number of elements ever placed onto the stack by the code section
    pub max_stack_size: u16,
}

impl TypesSection {
    /// Calculates the difference between the number of input and output stack elements.
    #[inline]
    pub const fn io_diff(&self) -> i32 {
        self.outputs as i32 - self.inputs as i32
    }

    /// Encode the section into the buffer.
    #[inline]
    pub fn encode(&self, buffer: &mut Vec<u8>) {
        buffer.push(self.inputs);
        buffer.push(self.outputs);
        buffer.extend_from_slice(&self.max_stack_size.to_be_bytes());
    }

    /// Decode the section from the input.
    #[inline]
    pub fn decode(input: &[u8]) -> Result<(Self, &[u8]), EofDecodeError> {
        let (input, inputs) = consume_u8(input)?;
        let (input, outputs) = consume_u8(input)?;
        let (input, max_stack_size) = consume_u16(input)?;
        let section = Self {
            inputs,
            outputs,
            max_stack_size,
        };
        section.validate()?;
        Ok((section, input))
    }

    /// Validate the section.
    pub fn validate(&self) -> Result<(), EofDecodeError> {
        if self.inputs > 0x7f || self.outputs > 0x80 || self.max_stack_size > 0x03FF {
            return Err(EofDecodeError::InvalidTypesSection);
        }
        if self.inputs as u16 > self.max_stack_size {
            return Err(EofDecodeError::InvalidTypesSection);
        }
        Ok(())
    }
}
