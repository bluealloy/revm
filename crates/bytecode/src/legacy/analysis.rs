use super::JumpTable;
use crate::opcode;
use bitvec::{bitvec, order::Lsb0, vec::BitVec};
use primitives::Bytes;
use std::vec::Vec;

/// Analyzes the bytecode to produce a jump table and potentially padded bytecode.
///
/// Prefer using [`Bytecode::new_legacy`](crate::Bytecode::new_legacy) instead.
pub(crate) fn analyze_legacy(bytecode: Bytes) -> (JumpTable, Bytes) {
    let mut jumps: BitVec<u8> = bitvec![u8, Lsb0; 0; bytecode.len()];
    let range = bytecode.as_ptr_range();
    let start = range.start;
    let mut iterator = start;
    let end = range.end;
    let mut last_byte: u8 = 0;

    while iterator < end {
        last_byte = unsafe { *iterator };
        if last_byte == opcode::JUMPDEST {
            // SAFETY: Jumps are max length of the code
            unsafe { jumps.set_unchecked(iterator.offset_from_unsigned(start), true) }
            iterator = unsafe { iterator.add(1) };
        } else {
            let push_offset = last_byte.wrapping_sub(opcode::PUSH1);
            if push_offset < 32 {
                // A trailing PUSH can advance the iterator past the end of the
                // bytecode allocation; `wrapping_add` keeps that offset
                // computation defined (the `< end` guard prevents any OOB read).
                iterator = iterator.wrapping_add(push_offset as usize + 2);
            } else if is_dupn_swapn_exchange(last_byte) {
                // Same as PUSH: skip the 1-byte immediate. Trailing opcodes may
                // advance past `end`; `wrapping_add` keeps that defined.
                iterator = iterator.wrapping_add(2);
            } else {
                // SAFETY: Iterator access range is checked in the while loop
                iterator = unsafe { iterator.add(1) };
            }
        }
    }

    // Calculate padding needed:
    // push_overflow covers incomplete PUSH / DUPN / SWAPN / EXCHANGE immediates
    // that caused the iterator to advance past the end of the bytecode.
    let push_overflow = (iterator as usize) - (end as usize);
    let mut padding = push_overflow;

    if last_byte != opcode::STOP {
        // Append a final STOP so execution always has a terminating opcode.
        padding += 1;
    }

    let bytecode = if padding > 0 {
        let mut padded = Vec::with_capacity(bytecode.len() + padding);
        padded.extend_from_slice(&bytecode);
        padded.resize(padded.len() + padding, 0);
        Bytes::from(padded)
    } else {
        bytecode
    };

    (JumpTable::new(jumps), bytecode)
}

/// Returns true if the opcode is DUPN, SWAPN, or EXCHANGE.
const fn is_dupn_swapn_exchange(opcode: u8) -> bool {
    opcode.wrapping_sub(opcode::DUPN) < 3
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn test_bytecode_with_jumpdest_at_start() {
        let bytecode = vec![opcode::JUMPDEST, opcode::PUSH1, 0x01, opcode::STOP];
        let (jump_table, _) = analyze_legacy(bytecode.into());
        assert!(jump_table.is_valid(0)); // First byte should be a valid jumpdest
    }

    #[test]
    fn test_bytecode_with_jumpdest_after_push() {
        let bytecode = vec![opcode::PUSH1, 0x01, opcode::JUMPDEST, opcode::STOP];
        let (jump_table, _) = analyze_legacy(bytecode.into());
        assert!(jump_table.is_valid(2)); // JUMPDEST should be at position 2
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
        let (jump_table, _) = analyze_legacy(bytecode.into());
        assert!(jump_table.is_valid(0)); // First JUMPDEST
        assert!(jump_table.is_valid(3)); // Second JUMPDEST
    }

    #[test]
    fn test_bytecode_with_max_push32() {
        let bytecode = vec![opcode::PUSH32];
        let (_, padded_bytecode) = analyze_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), bytecode.len() + 33); // PUSH32 + 32 bytes + STOP
    }

    #[test]
    fn test_truncated_pushes_are_padded_without_inbounds_pointer_advance() {
        for push in opcode::PUSH1..=opcode::PUSH32 {
            let bytecode = vec![push];
            let (_, padded_bytecode) = analyze_legacy(bytecode.clone().into());
            let push_immediate_len = (push - opcode::PUSH1 + 1) as usize;
            assert_eq!(
                padded_bytecode.len(),
                bytecode.len() + push_immediate_len + 1
            );
        }
    }

    #[test]
    fn test_bytecode_with_invalid_opcode() {
        let bytecode = vec![0xFF, opcode::STOP]; // 0xFF is an invalid opcode
        let (jump_table, _) = analyze_legacy(bytecode.into());
        assert!(!jump_table.is_valid(0)); // Invalid opcode should not be a jumpdest
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
        assert!(!jump_table.is_valid(0)); // PUSH1
        assert!(!jump_table.is_valid(2)); // PUSH2
        assert!(!jump_table.is_valid(5)); // PUSH4
    }

    #[test]
    fn test_bytecode_with_jumpdest_in_push_data() {
        let bytecode = vec![
            opcode::PUSH2,
            opcode::JUMPDEST, // This should not be treated as a JUMPDEST
            0x02,
            opcode::STOP,
        ];
        let (jump_table, _) = analyze_legacy(bytecode.into());
        assert!(!jump_table.is_valid(1)); // JUMPDEST in push data should not be valid
    }

    #[test]
    fn test_bytecode_ends_with_immediate_opcode_and_stop_requires_padding() {
        // For SWAPN/DUPN/EXCHANGE, the STOP (0x00) is consumed as the immediate operand,
        // not as an actual STOP instruction, so padding is needed.
        // [OPCODE]       -> [OPCODE, STOP, STOP] (3 bytes)
        // [OPCODE, STOP] -> [OPCODE, STOP, STOP] (3 bytes)
        for op in [opcode::SWAPN, opcode::DUPN, opcode::EXCHANGE] {
            for bytecode in [vec![op], vec![op, opcode::STOP]] {
                let (_, padded_bytecode) = analyze_legacy(bytecode.into());
                assert_eq!(padded_bytecode.len(), 3);
                assert_eq!(padded_bytecode[0], op);
                assert_eq!(padded_bytecode[1], opcode::STOP);
                assert_eq!(padded_bytecode[2], opcode::STOP);
            }
        }
    }

    #[test]
    fn test_jumpdest_in_dupn_swapn_exchange_immediate_is_not_valid() {
        // Regression: DUPN/SWAPN/EXCHANGE have a 1-byte immediate that must be
        // skipped during analysis (same as PUSH data). Otherwise a JUMPDEST byte
        // in the immediate is incorrectly marked as a valid jump target.
        for op in [opcode::DUPN, opcode::SWAPN, opcode::EXCHANGE] {
            let bytecode = vec![op, opcode::JUMPDEST, opcode::STOP];
            let (jump_table, padded_bytecode) = analyze_legacy(bytecode.clone().into());
            assert_eq!(padded_bytecode.len(), bytecode.len());
            assert!(
                !jump_table.is_valid(1),
                "immediate of {op:#04x} must not be JUMPDEST"
            );
        }
    }

    #[test]
    fn test_truncated_dupn_swapn_exchange_are_padded_like_push() {
        // Truncated opcode+immediate must pad the missing immediate and a STOP.
        for op in [opcode::DUPN, opcode::SWAPN, opcode::EXCHANGE] {
            let bytecode = vec![op];
            let (_, padded_bytecode) = analyze_legacy(bytecode.clone().into());
            assert_eq!(padded_bytecode.len(), bytecode.len() + 2);
            assert_eq!(&padded_bytecode[..], &[op, opcode::STOP, opcode::STOP]);
        }
    }
}
