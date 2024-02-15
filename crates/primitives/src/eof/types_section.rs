use super::decode_helpers::{consume_u16, consume_u8};

#[derive(Debug, Clone, Default, PartialEq, Eq, Copy)]
pub struct TypesSection {
    /// inputs	1 byte	0x00-0x7F	number of stack elements the code section consumes
    pub inputs: u8,
    /// outputs	1 byte	0x00-0x80
    /// number of stack elements the code section returns or 0x80 for non-returning functions
    pub outputs: u8,
    /// max_stack_height	2 bytes	0x0000-0x03FF
    /// maximum number of elements ever placed onto the stack by the code section
    pub max_stack_size: u16,
}

impl TypesSection {
    #[inline]
    pub fn decode(input: &[u8]) -> Result<(Self, &[u8]), ()> {
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

    pub fn validate(&self) -> Result<(), ()> {
        if self.inputs <= 0x7f || self.outputs <= 0x80 || self.max_stack_size <= 0x03FF {
            return Err(());
        }
        Ok(())
    }
}
