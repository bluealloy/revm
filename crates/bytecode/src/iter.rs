use crate::{opcode, Bytecode, OpCode};

/// Iterator over opcodes in a bytecode, skipping immediates.
///
/// This allows you to iterate through the actual opcodes in the bytecode,
/// without dealing with the immediate values that follow instructions.
#[derive(Debug, Clone)]
pub struct BytecodeIterator<'a> {
    /// Start pointer of the bytecode. Only used to calculate [`position`](Self::position).
    start: *const u8,
    /// Iterator over the bytecode bytes.
    bytes: core::slice::Iter<'a, u8>,
}

impl<'a> BytecodeIterator<'a> {
    /// Creates a new iterator from a bytecode reference.
    #[inline]
    pub fn new(bytecode: &'a Bytecode) -> Self {
        let bytes = match bytecode {
            Bytecode::LegacyAnalyzed(_) | Bytecode::Eof(_) => &bytecode.bytecode()[..],
            Bytecode::Eip7702(_) => &[],
        };
        Self {
            start: bytes.as_ptr(),
            bytes: bytes.iter(),
        }
    }

    /// Skips to the next opcode, taking into account PUSH instructions.
    pub fn skip_to_next_opcode(&mut self) {
        self.next();
    }

    /// Returns the remaining bytes in the bytecode as a slice.
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        self.bytes.as_slice()
    }

    /// Returns the current position in the bytecode.
    #[inline]
    pub fn position(&self) -> usize {
        (self.bytes.as_slice().as_ptr() as usize) - (self.start as usize)
        // TODO: Use the following on 1.87
        // SAFETY: `start` always points to the start of the bytecode.
        // unsafe {
        //     self.bytes
        //         .as_slice()
        //         .as_ptr()
        //         .offset_from_unsigned(self.start)
        // }
    }

    #[inline]
    fn skip_immediate(&mut self, opcode: u8) {
        // Get base immediate size from opcode info
        let mut immediate_size = opcode::OPCODE_INFO[opcode as usize]
            .map(|info| info.immediate_size() as usize)
            .unwrap_or_default();

        // Special handling for RJUMPV which has variable immediates
        if opcode == opcode::RJUMPV {
            if let Some(max_index) = self.peek() {
                immediate_size += (max_index as usize + 1) * 2;
            }
        }

        // Advance the iterator by the immediate size
        if immediate_size > 0 {
            self.bytes = self
                .bytes
                .as_slice()
                .get(immediate_size..)
                .unwrap_or_default()
                .iter();
        }
    }

    /// Returns the current opcode without advancing the iterator.
    #[inline]
    pub fn peek(&self) -> Option<u8> {
        self.bytes.as_slice().first().copied()
    }

    /// Returns the current opcode wrapped in OpCode without advancing the iterator.
    #[inline]
    pub fn peek_opcode(&self) -> Option<OpCode> {
        self.peek().and_then(OpCode::new)
    }
}

impl Iterator for BytecodeIterator<'_> {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.bytes
            .next()
            .copied()
            .inspect(|&current| self.skip_immediate(current))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // Lower bound is 0 if empty, 1 if not empty as it depends on the bytes.
        let byte_len = self.bytes.len();
        (byte_len.min(1), Some(byte_len))
    }
}

impl core::iter::FusedIterator for BytecodeIterator<'_> {}

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
    fn test_bytecode_skips_immediates() {
        // Create a bytecode with various PUSH operations
        let bytecode_data = vec![
            opcode::PUSH1,
            0x01, // PUSH1 0x01
            opcode::PUSH2,
            0x02,
            0x03,        // PUSH2 0x0203
            opcode::ADD, // ADD
            opcode::PUSH3,
            0x04,
            0x05,
            0x06, // PUSH3 0x040506
            opcode::PUSH32,
            0x10,
            0x11,
            0x12,
            0x13, // PUSH32 with 32 bytes of immediate data
            0x14,
            0x15,
            0x16,
            0x17,
            0x18,
            0x19,
            0x1a,
            0x1b,
            0x1c,
            0x1d,
            0x1e,
            0x1f,
            0x20,
            0x21,
            0x22,
            0x23,
            0x24,
            0x25,
            0x26,
            0x27,
            0x28,
            0x29,
            0x2a,
            0x2b,
            0x2c,
            0x2d,
            0x2e,
            0x2f,
            opcode::MUL,  // MUL
            opcode::STOP, // STOP
        ];

        let raw_bytecode = LegacyRawBytecode(Bytes::from(bytecode_data));
        let bytecode = Bytecode::LegacyAnalyzed(raw_bytecode.into_analyzed());

        // Use the iterator directly
        let iter = BytecodeIterator::new(&bytecode);
        let opcodes: Vec<u8> = iter.collect();

        // Should only include the opcodes, not the immediates
        assert_eq!(
            opcodes,
            vec![
                opcode::PUSH1,
                opcode::PUSH2,
                opcode::ADD,
                opcode::PUSH3,
                opcode::PUSH32,
                opcode::MUL,
                opcode::STOP,
            ]
        );

        // Use the method on the bytecode struct
        let opcodes: Vec<u8> = bytecode.iter_opcodes().collect();
        assert_eq!(
            opcodes,
            vec![
                opcode::PUSH1,
                opcode::PUSH2,
                opcode::ADD,
                opcode::PUSH3,
                opcode::PUSH32,
                opcode::MUL,
                opcode::STOP,
            ]
        );
    }

    #[test]
    fn test_position_tracking() {
        // PUSH1 0x01, PUSH1 0x02, ADD, STOP
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

        let mut iter = bytecode.iter_opcodes();

        // Start at position 0
        assert_eq!(iter.position(), 0);
        assert_eq!(iter.next(), Some(opcode::PUSH1));
        // After PUSH1, position should be 2 (PUSH1 + immediate)
        assert_eq!(iter.position(), 2);

        assert_eq!(iter.next(), Some(opcode::PUSH1));
        // After second PUSH1, position should be 4 (2 + PUSH1 + immediate)
        assert_eq!(iter.position(), 4);

        assert_eq!(iter.next(), Some(opcode::ADD));
        // After ADD, position should be 5 (4 + ADD)
        assert_eq!(iter.position(), 5);

        assert_eq!(iter.next(), Some(opcode::STOP));
        // After STOP, position should be 6 (5 + STOP)
        assert_eq!(iter.position(), 6);

        // No more opcodes
        assert_eq!(iter.next(), None);
        assert_eq!(iter.position(), 6);
    }

    #[test]
    fn test_empty_bytecode() {
        // Empty bytecode (just STOP)
        let bytecode_data = vec![opcode::STOP];
        let raw_bytecode = LegacyRawBytecode(Bytes::from(bytecode_data));
        let bytecode = Bytecode::LegacyAnalyzed(raw_bytecode.into_analyzed());

        let opcodes: Vec<u8> = bytecode.iter_opcodes().collect();
        assert_eq!(opcodes, vec![opcode::STOP]);
    }
}
