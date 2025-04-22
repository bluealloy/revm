use super::JumpTable;
use crate::opcode;
use bitvec::{bitvec, order::Lsb0, vec::BitVec};
use primitives::Bytes;
use std::{sync::Arc, vec, vec::Vec};

/// Analyze the bytecode to find the jumpdests. Used to create a jump table
/// that is needed for [`crate::LegacyAnalyzedBytecode`].
/// This function contains a hot loop and should be optimized as much as possible.
///
/// # Safety
///
/// The function uses unsafe pointer arithmetic, but maintains the following invariants:
/// - The iterator never advances beyond the end of the bytecode
/// - All pointer offsets are within bounds of the bytecode
/// - The jump table is never accessed beyond its allocated size
///
/// Undefined behavior if the bytecode does not end with a valid STOP opcode. Please check
/// [`crate::LegacyAnalyzedBytecode::new`] for details on how the bytecode is validated.
pub fn analyze_legacy(bytecode: Bytes) -> (JumpTable, Bytes) {
    if bytecode.is_empty() {
        return (JumpTable::default(), Bytes::from_static(&[opcode::STOP]));
    }

    let mut jumps: BitVec<u8> = bitvec![u8, Lsb0; 0; bytecode.len()];
    let range = bytecode.as_ptr_range();
    let start = range.start;
    let mut iterator = start;
    let end = range.end;
    let mut opcode = 0;

    while iterator < end {
        opcode = unsafe { *iterator };
        if opcode::JUMPDEST == opcode {
            // SAFETY: Jumps are max length of the code
            unsafe { jumps.set_unchecked(iterator.offset_from(start) as usize, true) }
            iterator = unsafe { iterator.offset(1) };
        } else {
            let push_offset = opcode.wrapping_sub(opcode::PUSH1);
            if push_offset < 32 {
                // SAFETY: Iterator access range is checked in the while loop
                iterator = unsafe { iterator.offset((push_offset + 2) as isize) };
            } else {
                // SAFETY: Iterator access range is checked in the while loop
                iterator = unsafe { iterator.offset(1) };
            }
        }
    }

    // Calculate padding needed to ensure bytecode ends with STOP
    // If we're at the end and last opcode is not STOP, we need 1 more byte
    let padding_size = (iterator as usize) - (end as usize) + (opcode != opcode::STOP) as usize;
    if padding_size > 0 {
        let mut padded_bytecode = Vec::with_capacity(bytecode.len() + padding_size);
        padded_bytecode.extend_from_slice(&bytecode);
        padded_bytecode.extend(vec![0; padding_size]);
        (JumpTable(Arc::new(jumps)), Bytes::from(padded_bytecode))
    } else {
        (JumpTable(Arc::new(jumps)), bytecode)
    }
}

mod tests {
    #[allow(unused_imports)]
    use crate::{legacy::analyze_legacy, opcode};

    #[test]
    fn test_bytecode_ends_with_stop_no_padding_needed() {
        let bytecode = vec![
            opcode::PUSH1,
            0x01,
            opcode::PUSH1,
            0x02,
            opcode::ADD,
            opcode::STOP,
        ];
        let (_, padded_bytecode) = analyze_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), bytecode.len());
    }

    #[test]
    fn test_bytecode_ends_without_stop_requires_padding() {
        let bytecode = vec![opcode::PUSH1, 0x01, opcode::PUSH1, 0x02, opcode::ADD];
        let (_, padded_bytecode) = analyze_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), bytecode.len() + 1);
    }

    #[test]
    fn test_bytecode_ends_with_push16_requires_17_bytes_padding() {
        let bytecode = vec![opcode::PUSH1, 0x01, opcode::PUSH16];
        let (_, padded_bytecode) = analyze_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), bytecode.len() + 17);
    }

    #[test]
    fn test_bytecode_ends_with_push2_requires_2_bytes_padding() {
        let bytecode = vec![opcode::PUSH1, 0x01, opcode::PUSH2, 0x02];
        let (_, padded_bytecode) = analyze_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), bytecode.len() + 2);
    }

    #[test]
    fn test_empty_bytecode_requires_stop() {
        let bytecode = vec![];
        let (_, padded_bytecode) = analyze_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), 1); // Just STOP
    }

    #[test]
    fn test_bytecode_with_jumpdest_at_start() {
        let bytecode = vec![opcode::JUMPDEST, opcode::PUSH1, 0x01, opcode::STOP];
        let (jump_table, _) = analyze_legacy(bytecode.clone().into());
        assert!(jump_table.0[0]); // First byte should be a valid jumpdest
    }

    #[test]
    fn test_bytecode_with_jumpdest_after_push() {
        let bytecode = vec![opcode::PUSH1, 0x01, opcode::JUMPDEST, opcode::STOP];
        let (jump_table, _) = analyze_legacy(bytecode.clone().into());
        assert!(jump_table.0[2]); // JUMPDEST should be at position 2
    }

    #[test]
    fn test_bytecode_with_multiple_jumpdests() {
        let bytecode = vec![
            opcode::JUMPDEST,
            opcode::PUSH1,
            0x01,
            opcode::JUMPDEST,
            opcode::STOP,
        ];
        let (jump_table, _) = analyze_legacy(bytecode.clone().into());
        assert!(jump_table.0[0]); // First JUMPDEST
        assert!(jump_table.0[3]); // Second JUMPDEST
    }

    #[test]
    fn test_bytecode_with_max_push32() {
        let bytecode = vec![opcode::PUSH32];
        let (_, padded_bytecode) = analyze_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), bytecode.len() + 33); // PUSH32 + 32 bytes + STOP
    }

    #[test]
    fn test_bytecode_with_invalid_opcode() {
        let bytecode = vec![0xFF, opcode::STOP]; // 0xFF is an invalid opcode
        let (jump_table, _) = analyze_legacy(bytecode.clone().into());
        assert!(!jump_table.0[0]); // Invalid opcode should not be a jumpdest
    }

    #[test]
    fn test_bytecode_with_sequential_pushes() {
        let bytecode = vec![
            opcode::PUSH1,
            0x01,
            opcode::PUSH2,
            0x02,
            0x03,
            opcode::PUSH4,
            0x04,
            0x05,
            0x06,
            0x07,
            opcode::STOP,
        ];
        let (jump_table, padded_bytecode) = analyze_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), bytecode.len());
        assert!(!jump_table.0[0]); // PUSH1
        assert!(!jump_table.0[2]); // PUSH2
        assert!(!jump_table.0[5]); // PUSH4
    }

    #[test]
    fn test_bytecode_with_jumpdest_in_push_data() {
        let bytecode = vec![
            opcode::PUSH2,
            opcode::JUMPDEST, // This should not be treated as a JUMPDEST
            0x02,
            opcode::STOP,
        ];
        let (jump_table, _) = analyze_legacy(bytecode.clone().into());
        assert!(!jump_table.0[1]); // JUMPDEST in push data should not be valid
    }
}
