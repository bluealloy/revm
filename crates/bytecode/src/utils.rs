//! Various utilities for the bytecode

/// Reads a big-endian `i16` from a `u8` pointer.
///
/// # Safety
///
/// The pointer must point to at least 2 bytes.
#[inline]
pub unsafe fn read_i16(ptr: *const u8) -> i16 {
    read_u16(ptr) as i16
}

/// Reads a big-endian `u16` from a `u8` pointer.
///
/// # Safety
///
/// The pointer must point to at least 2 bytes.
#[inline]
pub unsafe fn read_u16(ptr: *const u8) -> u16 {
    u16::from_be_bytes(unsafe { ptr.cast::<[u8; 2]>().read() })
}

/// Bytecode test utilities
#[cfg(test)]
pub mod test {
    use crate::opcode;
    use alloy_primitives::{hex, Bytes};
    use anyhow::Result;
    use std::str::FromStr;

    /// Constructs bytecode for inserting input into memory
    pub fn build_memory_input_opcodes(hex_input: &str) -> Result<Vec<u8>> {
        let mut opcodes = vec![];

        // Parse hex input string to bytes
        let input_bytes = Bytes::from_str(hex_input)?;

        // Iterate for each 32 bytes to prepend PUSH* and append MSTORE opcodes
        for chunk in input_bytes.chunks(32) {
            let push_opcode = opcode::PUSH0 + chunk.len() as u8;
            opcodes.push(push_opcode);

            opcodes.extend_from_slice(chunk);

            opcodes.push(opcode::MSTORE);
        }

        Ok(opcodes)
    }

    #[test]
    fn test_build_memory_input_opcodes() {
        let mut bytes = vec![];
        bytes.extend([0xff; 32]);
        bytes.extend([0x77; 32]);
        bytes.extend([0x11; 32]);
        bytes.extend([1, 2, 3, 4]);

        let hex_input = hex::encode(bytes);

        let opcodes = build_memory_input_opcodes(&hex_input).unwrap();
        let mut expected = vec![];
        expected.push(opcode::PUSH32);
        expected.extend([0xff; 32]);
        expected.push(opcode::MSTORE);
        expected.push(opcode::PUSH32);
        expected.extend([0x77; 32]);
        expected.push(opcode::MSTORE);
        expected.push(opcode::PUSH32);
        expected.extend([0x11; 32]);
        expected.push(opcode::MSTORE);
        expected.push(opcode::PUSH4);
        expected.extend([1, 2, 3, 4]);
        expected.push(opcode::MSTORE);

        assert_eq!(opcodes, expected);
    }
}
