use crate::{opcode, Bytecode, OpCode};

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
            Bytecode::Eip7702(_) => 0,
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

        if opcode::OpCode::new(opcode).is_none() {
            // Unknown opcode, return in that case
            return;
        }

        // Special case: RJUMPV has a variable number of immediates
        if opcode == opcode::RJUMPV {
            if self.position < self.end {
                // The first immediate byte is the max_index
                let max_index = self.bytes[self.position] as usize;
                let rjumpv_additional_immediates = (max_index + 1) * 2; // Including the max_index byte itself

                // Skip the immediate bytes
                self.position =
                    core::cmp::min(self.position + 1 + rjumpv_additional_immediates, self.end);
            }
        } else {
            // For all other opcodes, use the immediate_size from OPCODE_INFO
            if let Some(opcode_info) = opcode::OPCODE_INFO[opcode as usize] {
                let immediate_size = opcode_info.immediate_size() as usize;
                if immediate_size > 0 {
                    self.position = core::cmp::min(self.position + immediate_size, self.end);
                }
            }
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
    use crate::{eof::Eof, LegacyRawBytecode};
    use primitives::{Address, Bytes};

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

    #[test]
    fn test_eip7702_bytecode_iteration() {
        // Create a simple Eip7702 bytecode with address
        let address = Address::new([0x42; 20]);
        let bytecode = Bytecode::new_eip7702(address);

        // Verify no opcodes are returned when iterating
        let opcodes: Vec<u8> = bytecode.iter_opcodes().collect();
        assert!(
            opcodes.is_empty(),
            "No opcodes should be returned for Eip7702 bytecode"
        );

        // Verify peek returns None immediately
        #[warn(unused_mut)]
        let mut iter = bytecode.iter_opcodes();
        assert_eq!(
            iter.peek(),
            None,
            "Peek should return None for Eip7702 bytecode"
        );
    }

    #[test]
    fn test_eof_opcodes_with_immediates() {
        // Test with some EOF opcodes that have immediate values
        let bytecode_data = vec![
            opcode::RJUMP,
            0x01,
            0x02, // RJUMP with 2 immediate bytes
            opcode::DATALOADN,
            0x03,
            0x04, // DATALOADN with 2 immediate bytes
            opcode::RJUMPV,
            0x01, // RJUMPV with max_index=1 (2 entries in table)
            0x05,
            0x06,
            0x07,
            0x08, // Jump table with 2 entries (4 bytes)
            opcode::CALLF,
            0x09,
            0x0A,         
            opcode::STOP, 
        ];

        // Create a mock bytecode with the data
        let mut iter = BytecodeIterator {
            bytes: &bytecode_data,
            position: 0,
            end: bytecode_data.len(),
        };

        // Check RJUMP
        assert_eq!(iter.next(), Some(opcode::RJUMP));

        // Check DATALOADN
        assert_eq!(iter.next(), Some(opcode::DATALOADN));

        // Check RJUMPV 
        assert_eq!(iter.next(), Some(opcode::RJUMPV));

        // Check CALLF
        assert_eq!(iter.next(), Some(opcode::CALLF));

        // Check STOP
        assert_eq!(iter.next(), Some(opcode::STOP));

        assert_eq!(iter.next(), None);
    }
}
