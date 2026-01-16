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
    use anyhow::Result;
    use primitives::U256;
    use rand::Rng;

    /// Constructs bytecode for inserting input into memory
    pub fn build_memory_input_opcodes(start_offset: U256, input: &[u8]) -> Result<Vec<u8>> {
        let mut opcodes = vec![];
        let mut current_offset = start_offset;

        // Iterate for each 32 bytes to prepend PUSH* and append MSTORE opcodes
        let offset_step = U256::from(32);
        for bytes in input.chunks(32) {
            // Push the input value
            build_push_bytes(bytes, &mut opcodes);

            // Push the memory offset
            build_push_u256(current_offset, &mut opcodes);

            // Call MSTORE
            opcodes.push(opcode::MSTORE);

            // Increase the memory offset
            current_offset += offset_step;
        }

        Ok(opcodes)
    }

    // Constructs a PUSH* instruction for an Uint256
    fn build_push_u256(value: U256, opcodes: &mut Vec<u8>) {
        let bytes = value.to_be_bytes_trimmed_vec();
        build_push_bytes(&bytes, opcodes);
    }

    // Constructs a PUSH* instruction for the value of byte size is not greater than 32
    fn build_push_bytes(bytes: &[u8], opcodes: &mut Vec<u8>) {
        let len = bytes.len();
        assert!(len <= 32);

        let push_opcode = opcode::PUSH0 + len as u8;
        opcodes.push(push_opcode);

        opcodes.extend_from_slice(bytes);
    }

    #[test]
    fn test_build_memory_input_opcodes() {
        let mut rng = rand::rng();

        // make the memory offset as 4 bytes for test
        let start_offset = rng.random_range(0x0100_0000..=(u32::MAX - 100));
        let mut current_offset = start_offset;

        let mut all_inputs = vec![];
        let mut expected_opcodes = vec![];

        // Generate 32 bytes input array
        let input_arr: [[u8; 32]; 3] = rng.random();
        for input in input_arr {
            all_inputs.extend(input);

            expected_opcodes.push(opcode::PUSH32);
            expected_opcodes.extend(input);
            expected_opcodes.push(opcode::PUSH4);
            expected_opcodes.extend(current_offset.to_be_bytes());
            expected_opcodes.push(opcode::MSTORE);

            current_offset += 32;
        }

        let last_input: [u8; 15] = rng.random();
        {
            all_inputs.extend(last_input);

            expected_opcodes.push(opcode::PUSH15);
            expected_opcodes.extend(last_input);
            expected_opcodes.push(opcode::PUSH4);
            expected_opcodes.extend(current_offset.to_be_bytes());

            expected_opcodes.push(opcode::MSTORE);
        }

        let opcodes = build_memory_input_opcodes(U256::from(start_offset), &all_inputs).unwrap();
        assert_eq!(opcodes, expected_opcodes);
    }
}
