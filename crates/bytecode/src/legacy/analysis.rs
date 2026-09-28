use super::JumpTable;
use crate::opcode;
use bitvec::{bitvec, order::Lsb0, vec::BitVec};
use primitives::Bytes;
use std::vec::Vec;

/// Analyzes the original bytecode to produce a jump table.
pub(crate) fn analyze_legacy(bytecode: &[u8]) -> JumpTable {
    let mut jumps: BitVec<u8> = bitvec![u8, Lsb0; 0; bytecode.len()];
    let range = bytecode.as_ptr_range();
    let start = range.start;
    let mut iterator = start;
    let end = range.end;

    while iterator < end {
        let last_byte = unsafe { *iterator };
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
            } else {
                // SAFETY: Iterator access range is checked in the while loop
                iterator = unsafe { iterator.add(1) };
            }
        }
    }

    JumpTable::new(jumps)
}

/// Maximum PUSH immediate length plus a terminating STOP.
const PADDING: usize = 33;

/// Appends zero padding unless the bytecode already ends with 33 zeros.
pub(crate) fn pad_legacy(bytecode: Bytes) -> Bytes {
    if bytecode.is_empty() {
        return Bytes::from_static(&[opcode::STOP]);
    }
    if bytecode.ends_with(&[0; PADDING]) {
        return bytecode;
    }

    let padded_len = bytecode.len() + PADDING;
    match bytecode.0.try_into_mut() {
        Ok(mut bytecode) => {
            bytecode.resize(padded_len, 0);
            bytecode.freeze().into()
        }
        Err(bytecode) => {
            let mut padded = Vec::with_capacity(padded_len);
            padded.extend_from_slice(&bytecode);
            padded.resize(padded_len, 0);
            padded.into()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec;

    #[test]
    fn test_bytecode_ends_with_stop_still_padded() {
        let bytecode = vec![
            opcode::PUSH1,
            0x01,
            opcode::PUSH1,
            0x02,
            opcode::ADD,
            opcode::STOP,
        ];
        let padded_bytecode = pad_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), bytecode.len() + 33);
    }

    #[test]
    fn test_bytecode_ends_without_stop_requires_padding() {
        let bytecode = vec![opcode::PUSH1, 0x01, opcode::PUSH1, 0x02, opcode::ADD];
        let padded_bytecode = pad_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), bytecode.len() + 33);
    }

    #[test]
    fn test_bytecode_ends_with_push16() {
        let bytecode = vec![opcode::PUSH1, 0x01, opcode::PUSH16];
        let padded_bytecode = pad_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), bytecode.len() + 33);
    }

    #[test]
    fn test_bytecode_ends_with_push2() {
        let bytecode = vec![opcode::PUSH1, 0x01, opcode::PUSH2, 0x02];
        let padded_bytecode = pad_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), bytecode.len() + 33);
    }

    #[test]
    fn test_bytecode_with_jumpdest_at_start() {
        let bytecode = vec![opcode::JUMPDEST, opcode::PUSH1, 0x01, opcode::STOP];
        let jump_table = analyze_legacy(&bytecode);
        assert!(jump_table.is_valid(0)); // First byte should be a valid jumpdest
    }

    #[test]
    fn test_bytecode_with_jumpdest_after_push() {
        let bytecode = vec![opcode::PUSH1, 0x01, opcode::JUMPDEST, opcode::STOP];
        let jump_table = analyze_legacy(&bytecode);
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
        let jump_table = analyze_legacy(&bytecode);
        assert!(jump_table.is_valid(0)); // First JUMPDEST
        assert!(jump_table.is_valid(3)); // Second JUMPDEST
    }

    #[test]
    fn test_bytecode_with_max_push32() {
        let bytecode = vec![opcode::PUSH32];
        let padded_bytecode = pad_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), bytecode.len() + 33); // PUSH32 + 32 bytes + STOP
    }

    #[test]
    fn test_truncated_pushes_are_padded_without_inbounds_pointer_advance() {
        for push in opcode::PUSH1..=opcode::PUSH32 {
            let bytecode = vec![push];
            let padded_bytecode = pad_legacy(bytecode.clone().into());
            let push_immediate_len = (push - opcode::PUSH1 + 1) as usize;
            assert_eq!(padded_bytecode.len(), bytecode.len() + 33);
            assert!(padded_bytecode.len() > bytecode.len() + push_immediate_len);
        }
    }

    #[test]
    fn test_bytecode_with_invalid_opcode() {
        let bytecode = vec![0xFF, opcode::STOP]; // 0xFF is an invalid opcode
        let jump_table = analyze_legacy(&bytecode);
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
        let jump_table = analyze_legacy(&bytecode);
        let padded_bytecode = pad_legacy(bytecode.clone().into());
        assert_eq!(padded_bytecode.len(), bytecode.len() + 33);
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
        let jump_table = analyze_legacy(&bytecode);
        assert!(!jump_table.is_valid(1)); // JUMPDEST in push data should not be valid
    }

    #[test]
    fn test_bytecode_ends_with_immediate_opcode_and_stop_requires_padding() {
        // For SWAPN/DUPN/EXCHANGE, the STOP (0x00) is consumed as the immediate operand,
        // not as an actual STOP instruction, so padding is needed.
        // The fixed padding supplies both the immediate and a terminating STOP.
        for op in [opcode::SWAPN, opcode::DUPN, opcode::EXCHANGE] {
            for bytecode in [vec![op], vec![op, opcode::STOP]] {
                let original_len = bytecode.len();
                let padded_bytecode = pad_legacy(bytecode.into());
                assert_eq!(padded_bytecode.len(), original_len + 33);
                assert_eq!(padded_bytecode[0], op);
                assert_eq!(padded_bytecode[1], opcode::STOP);
                assert_eq!(padded_bytecode[2], opcode::STOP);
            }
        }
    }

    #[test]
    fn padding_zero_suffix_boundary() {
        for len in [1, 32, 33, 34, 65] {
            let raw = Bytes::from(vec![0; len]);
            let padded = pad_legacy(raw.clone());
            if len >= 33 {
                assert_eq!(padded.len(), len);
                assert_eq!(padded.as_ptr(), raw.as_ptr());
            } else {
                assert_eq!(padded.len(), len + 33);
            }
            assert!(padded.iter().all(|&byte| byte == 0));
        }

        // Every byte of the suffix must be zero to skip padding.
        for nonzero in 0..33 {
            let mut raw = vec![0; 33];
            raw[nonzero] = opcode::JUMPDEST;
            let padded = pad_legacy(raw.clone().into());
            assert_eq!(padded.len(), 66);
            assert_eq!(&padded[..33], &raw);
            assert_eq!(&padded[33..], &[0; 33]);
        }
    }

    #[test]
    fn padding_reuses_owned_capacity() {
        let mut raw = Vec::with_capacity(34);
        raw.push(opcode::PUSH32);
        let raw = Bytes::from(raw);
        let ptr = raw.as_ptr();
        let padded = pad_legacy(raw);
        assert_eq!(padded.as_ptr(), ptr);
        assert_eq!(padded.len(), 34);
        assert_eq!(&padded[1..], &[0; 33]);
    }

    #[test]
    fn padding_releases_shared_input() {
        let raw = Bytes::copy_from_slice(&[opcode::PUSH32]);
        let padded = pad_legacy(raw.clone());
        assert!(raw.is_unique());
        assert_ne!(padded.as_ptr(), raw.as_ptr());
        assert_eq!(&raw[..], &[opcode::PUSH32]);
        assert_eq!(padded.len(), 34);
        assert_eq!(&padded[1..], &[0; 33]);
    }
}
