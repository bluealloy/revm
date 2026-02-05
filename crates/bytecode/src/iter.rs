use crate::{opcode, Bytecode, OpCode};

/// Iterator over opcodes in a bytecode, skipping immediates.
///
/// This allows you to iterate through the actual opcodes in the bytecode,
/// without dealing with the immediate values that follow instructions.
#[derive(Debug, Clone)]
pub struct BytecodeIterator<'a> {
    /// Iterator over the bytecode bytes.
    bytes: core::slice::Iter<'a, u8>,
    /// Start pointer of the bytecode. Only used to calculate [`position`](Self::position).
    start: *const u8,
}

impl<'a> BytecodeIterator<'a> {
    /// Creates a new iterator from a bytecode reference.
    #[inline]
    pub fn new(bytecode: &'a Bytecode) -> Self {
        let bytes = if bytecode.is_legacy() {
            &bytecode.bytecode()[..]
        } else {
            &[]
        };
        Self {
            bytes: bytes.iter(),
            start: bytes.as_ptr(),
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
        // SAFETY: `start` always points to the start of the bytecode.
        unsafe {
            self.bytes
                .as_slice()
                .as_ptr()
                .offset_from_unsigned(self.start)
        }
    }

    #[inline]
    fn skip_immediate(&mut self, opcode: u8) {
        // Get base immediate size from opcode info
        let immediate_size = opcode::OPCODE_INFO[opcode as usize]
            .map(|info| info.immediate_size() as usize)
            .unwrap_or_default();

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
    use primitives::Bytes;

    #[test]
    fn test_simple_bytecode_iteration() {
        // Create a simple bytecode: PUSH1 0x01 PUSH1 0x02 ADD STOP
        let bytecode = Bytecode::new_legacy(Bytes::from_static(&[
            opcode::PUSH1,
            0x01,
            opcode::PUSH1,
            0x02,
            opcode::ADD,
            opcode::STOP,
        ]));
        let opcodes: Vec<u8> = bytecode.iter_opcodes().collect();
        assert_eq!(
            opcodes,
            vec![opcode::PUSH1, opcode::PUSH1, opcode::ADD, opcode::STOP]
        );
    }

    #[test]
    fn test_bytecode_with_various_push_sizes() {
        let bytecode = Bytecode::new_legacy(Bytes::from_static(&[
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
        ]));

        let opcodes: Vec<u8> = bytecode.iter_opcodes().collect();

        // We should only see the opcodes, not the immediates
        assert_eq!(
            opcodes,
            vec![opcode::PUSH1, opcode::PUSH2, opcode::PUSH3, opcode::STOP]
        );
    }

    #[test]
    fn test_bytecode_skips_immediates() {
        let bytecode = Bytecode::new_legacy(Bytes::from_static(&[
            opcode::PUSH1,
            0x01,
            opcode::PUSH2,
            0x02,
            0x03,
            opcode::ADD,
            opcode::PUSH3,
            0x04,
            0x05,
            0x06,
            opcode::PUSH32,
            0x10,
            0x11,
            0x12,
            0x13,
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
            opcode::MUL,
            opcode::STOP,
        ]));

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
        let bytecode = Bytecode::new_legacy(Bytes::from_static(&[
            opcode::PUSH1,
            0x01,
            opcode::PUSH1,
            0x02,
            opcode::ADD,
            opcode::STOP,
        ]));

        let mut iter = bytecode.iter_opcodes();

        assert_eq!(iter.position(), 0);
        assert_eq!(iter.next(), Some(opcode::PUSH1));
        assert_eq!(iter.position(), 2);

        assert_eq!(iter.next(), Some(opcode::PUSH1));
        assert_eq!(iter.position(), 4);

        assert_eq!(iter.next(), Some(opcode::ADD));
        assert_eq!(iter.position(), 5);

        assert_eq!(iter.next(), Some(opcode::STOP));
        assert_eq!(iter.position(), 6);

        assert_eq!(iter.next(), None);
        assert_eq!(iter.position(), 6);
    }

    #[test]
    fn test_empty_bytecode() {
        let bytecode = Bytecode::new_legacy(Bytes::from_static(&[opcode::STOP]));
        let opcodes: Vec<u8> = bytecode.iter_opcodes().collect();
        assert_eq!(opcodes, vec![opcode::STOP]);
    }
}
