#[cfg(test)]
mod bytecode_iterator_tests {
    use crate::{opcode, Bytecode, BytecodeIterator, LegacyRawBytecode, OpCode};
    use primitives::Bytes;

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
    fn test_peek_opcode() {
        // PUSH1 0x01, ADD, MUL, STOP
        let bytecode_data = vec![opcode::PUSH1, 0x01, opcode::ADD, opcode::MUL, opcode::STOP];
        let raw_bytecode = LegacyRawBytecode(Bytes::from(bytecode_data));
        let bytecode = Bytecode::LegacyAnalyzed(raw_bytecode.into_analyzed());

        let mut iter = bytecode.iter_opcodes();

        // Peek should return PUSH1
        assert_eq!(iter.peek(), Some(opcode::PUSH1));
        assert_eq!(
            iter.peek_opcode(),
            Some(OpCode::new(opcode::PUSH1).unwrap())
        );

        // Next should consume PUSH1
        assert_eq!(iter.next(), Some(opcode::PUSH1));

        // Peek should now return ADD
        assert_eq!(iter.peek(), Some(opcode::ADD));
        assert_eq!(iter.peek_opcode(), Some(OpCode::new(opcode::ADD).unwrap()));

        // Consume ADD
        assert_eq!(iter.next(), Some(opcode::ADD));

        // Peek should now return MUL
        assert_eq!(iter.peek(), Some(opcode::MUL));
        assert_eq!(iter.peek_opcode(), Some(OpCode::new(opcode::MUL).unwrap()));

        // Consume MUL and STOP
        assert_eq!(iter.next(), Some(opcode::MUL));
        assert_eq!(iter.next(), Some(opcode::STOP));

        // No more opcodes
        assert_eq!(iter.peek(), None);
        assert_eq!(iter.peek_opcode(), None);
        assert_eq!(iter.next(), None);
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
