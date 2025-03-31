use crate::{opcode, Bytecode, OpCode};
use std::cmp;
use core::cmp;



/// Iterator over opcodes in a bytecode, skipping immediates.
///
/// This allows you to iterate through the actual opcodes in the bytecode,
/// without dealing with the immediate values that follow PUSH instructions.
#[derive(Debug, Clone)]
pub struct BytecodeIterator<'a> {
    /// Reference to the underlying bytecode bytes
    bytes: &'a [u8],
    /// Current position in the bytecode
    position: usize,
    /// End position in the bytecode (to handle original length for legacy bytecode)
    end: usize,
}

impl<'a> BytecodeIterator<'a> {
    /// Creates a new iterator from a bytecode reference.
    pub fn new(bytecode: &'a Bytecode) -> Self {
        let bytes = bytecode.bytecode();
        let end = match bytecode {
            Bytecode::LegacyAnalyzed(analyzed) => analyzed.original_len(),
            _ => bytes.len(),
        };

        Self {
            bytes: bytes.as_ref(),
            position: 0,
            end,
        }
    }

    /// Returns the current position in the bytecode.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Skips to the next opcode, taking into account PUSH instructions.
    pub fn skip_to_next_opcode(&mut self) {
        if self.position >= self.end {
            return;
        }

        let opcode = self.bytes[self.position];
        self.position += 1;

        // If the opcode is PUSH1..PUSH32, skip the immediate bytes
        let push_offset = opcode.wrapping_sub(opcode::PUSH1);
        if push_offset < 32 {
            // Skip the immediate bytes (push_offset + 1 bytes)
            let immediate_size = push_offset as usize + 1;
            self.position = cmp::min(self.position + immediate_size, self.end);
        }
    }

    /// Returns the current opcode without advancing the iterator.
    pub fn peek(&self) -> Option<u8> {
        if self.position < self.end {
            Some(self.bytes[self.position])
        } else {
            None
        }
    }

    /// Returns the current opcode wrapped in OpCode without advancing the iterator.
    pub fn peek_opcode(&self) -> Option<OpCode> {
        self.peek().and_then(OpCode::new)
    }
}

#[allow(clippy::needless_lifetimes)]
impl<'a> Iterator for BytecodeIterator<'a> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= self.end {
            return None;
        }

        let opcode = self.bytes[self.position];
        self.skip_to_next_opcode();
        Some(opcode)
    }
}

/// Extension trait for Bytecode to provide iteration capabilities.
pub trait BytecodeIteratorExt {
    /// Returns an iterator over the opcodes in this bytecode, skipping immediates.
    fn iter_opcodes(&self) -> BytecodeIterator<'_>;
}

impl BytecodeIteratorExt for Bytecode {
    fn iter_opcodes(&self) -> BytecodeIterator<'_> {
        BytecodeIterator::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LegacyRawBytecode;
    use primitives::Bytes;

    #[test]
    fn test_simple_bytecode_iteration() {
        // Create a simple bytecode: PUSH1 0x01 PUSH1 0x02 ADD STOP
        let bytecode_data = vec![
            opcode::PUSH1,
            0x01,
            opcode::PUSH1,
            0x02,
            opcode::ADD,
            opcode::STOP,
        ];
        let raw_bytecode = LegacyRawBytecode(Bytes::from(bytecode_data));
        let bytecode = Bytecode::LegacyAnalyzed(raw_bytecode.into_analyzed());

        let opcodes: Vec<u8> = bytecode.iter_opcodes().collect();

        // We should only see the opcodes, not the immediates
        assert_eq!(
            opcodes,
            vec![opcode::PUSH1, opcode::PUSH1, opcode::ADD, opcode::STOP]
        );
    }

    #[test]
    fn test_bytecode_with_various_push_sizes() {
        // PUSH1 0x01, PUSH2 0x0203, PUSH3 0x040506, STOP
        let bytecode_data = vec![
            opcode::PUSH1,
            0x01,
            opcode::PUSH2,
            0x02,
            0x03,
            opcode::PUSH3,
            0x04,
            0x05,
            0x06,
            opcode::STOP,
        ];
        let raw_bytecode = LegacyRawBytecode(Bytes::from(bytecode_data));
        let bytecode = Bytecode::LegacyAnalyzed(raw_bytecode.into_analyzed());

        let opcodes: Vec<u8> = bytecode.iter_opcodes().collect();

        // We should only see the opcodes, not the immediates
        assert_eq!(
            opcodes,
            vec![opcode::PUSH1, opcode::PUSH2, opcode::PUSH3, opcode::STOP]
        );
    }

    #[test]
    fn test_peek_functionality() {
        // PUSH1 0x01, ADD, STOP
        let bytecode_data = vec![opcode::PUSH1, 0x01, opcode::ADD, opcode::STOP];
        let raw_bytecode = LegacyRawBytecode(Bytes::from(bytecode_data));
        let bytecode = Bytecode::LegacyAnalyzed(raw_bytecode.into_analyzed());

        let mut iter = bytecode.iter_opcodes();

        assert_eq!(iter.peek(), Some(opcode::PUSH1));
        assert_eq!(iter.next(), Some(opcode::PUSH1)); // This consumes PUSH1 and skips 0x01

        assert_eq!(iter.peek(), Some(opcode::ADD));
        assert_eq!(iter.next(), Some(opcode::ADD));

        assert_eq!(iter.peek(), Some(opcode::STOP));
        assert_eq!(iter.next(), Some(opcode::STOP));

        assert_eq!(iter.peek(), None);
        assert_eq!(iter.next(), None);
    }
}
