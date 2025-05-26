use super::JumpTable;
use crate::opcode;
use primitives::Bytes;

/// Legacy analyzed bytecode represents the original bytecode format used in Ethereum.
///
/// # Jump Table
///
/// A jump table maps valid jump destinations in the bytecode.
///
/// While other EVM implementations typically analyze bytecode and cache jump tables at runtime,
/// Revm requires the jump table to be pre-computed and contained alongside the code,
/// and present with the bytecode when executing.
///
/// # Bytecode Padding
///
/// All legacy bytecode is padded with 33 zero bytes at the end. This padding ensures the
/// bytecode always ends with a valid STOP (0x00) opcode. The reason for 33 bytes padding (and not one byte)
/// is handling the edge cases  where a PUSH32 opcode appears at the end of the original
/// bytecode without enough remaining bytes for its immediate data. Original bytecode length
/// is stored in order to be able to copy original bytecode.
///
/// # Gas safety
///
/// When bytecode is created through CREATE, CREATE2, or contract creation transactions, it undergoes
/// analysis to generate its jump table. This analysis is O(n) on side of bytecode that is expensive,
/// but the high gas cost required to store bytecode in the database is high enough to cover the
/// expense of doing analysis and generate the jump table.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LegacyAnalyzedBytecode {
    /// Bytecode with 33 zero bytes padding
    bytecode: Bytes,
    /// Original bytes length
    original_len: usize,
    /// Jump table
    jump_table: JumpTable,
}

impl Default for LegacyAnalyzedBytecode {
    #[inline]
    fn default() -> Self {
        Self {
            bytecode: Bytes::from_static(&[0]),
            original_len: 0,
            jump_table: JumpTable::default(),
        }
    }
}

impl LegacyAnalyzedBytecode {
    /// Creates new analyzed bytecode.
    ///
    /// # Panics
    ///
    /// * If `original_len` is greater than `bytecode.len()`
    /// * If jump table length is less than `original_len`.
    /// * If last bytecode byte is not `0x00` or if bytecode is empty.
    pub fn new(bytecode: Bytes, original_len: usize, jump_table: JumpTable) -> Self {
        if original_len > bytecode.len() {
            panic!("original_len is greater than bytecode length");
        }
        if original_len > jump_table.0.len() {
            panic!(
                "jump table length {} is less than original length {}",
                jump_table.0.len(),
                original_len
            );
        }

        if bytecode.is_empty() {
            panic!("bytecode cannot be empty");
        }

        if bytecode.last() != Some(&opcode::STOP) {
            panic!("last bytecode byte should be STOP (0x00)");
        }

        Self {
            bytecode,
            original_len,
            jump_table,
        }
    }

    /// Returns a reference to the bytecode.
    ///
    /// The bytecode is padded with 32 zero bytes.
    pub fn bytecode(&self) -> &Bytes {
        &self.bytecode
    }

    /// Returns original bytes length.
    pub fn original_len(&self) -> usize {
        self.original_len
    }

    /// Returns original bytes without padding.
    pub fn original_bytes(&self) -> Bytes {
        self.bytecode.slice(..self.original_len)
    }

    /// Returns original bytes without padding.
    pub fn original_byte_slice(&self) -> &[u8] {
        &self.bytecode[..self.original_len]
    }

    /// Returns [JumpTable] of analyzed bytes.
    pub fn jump_table(&self) -> &JumpTable {
        &self.jump_table
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{opcode, LegacyRawBytecode};
    use bitvec::{bitvec, order::Lsb0};
    use std::sync::Arc;

    #[test]
    fn test_bytecode_new() {
        let bytecode = Bytes::from_static(&[opcode::PUSH1, 0x01]);
        let bytecode = LegacyRawBytecode(bytecode).into_analyzed();
        let _ = LegacyAnalyzedBytecode::new(
            bytecode.bytecode,
            bytecode.original_len,
            bytecode.jump_table,
        );
    }

    #[test]
    #[should_panic(expected = "original_len is greater than bytecode length")]
    fn test_panic_on_large_original_len() {
        let bytecode = Bytes::from_static(&[opcode::PUSH1, 0x01]);
        let bytecode = LegacyRawBytecode(bytecode).into_analyzed();
        let _ = LegacyAnalyzedBytecode::new(bytecode.bytecode, 100, bytecode.jump_table);
    }

    #[test]
    #[should_panic(expected = "jump table length 1 is less than original length 2")]
    fn test_panic_on_short_jump_table() {
        let bytecode = Bytes::from_static(&[opcode::PUSH1, 0x01]);
        let bytecode = LegacyRawBytecode(bytecode).into_analyzed();
        let jump_table = JumpTable(Arc::new(bitvec![u8, Lsb0; 0; 1]));
        let _ = LegacyAnalyzedBytecode::new(bytecode.bytecode, bytecode.original_len, jump_table);
    }

    #[test]
    #[should_panic(expected = "last bytecode byte should be STOP (0x00)")]
    fn test_panic_on_non_stop_bytecode() {
        let bytecode = Bytes::from_static(&[opcode::PUSH1, 0x01]);
        let jump_table = JumpTable(Arc::new(bitvec![u8, Lsb0; 0; 2]));
        let _ = LegacyAnalyzedBytecode::new(bytecode, 2, jump_table);
    }

    #[test]
    #[should_panic(expected = "bytecode cannot be empty")]
    fn test_panic_on_empty_bytecode() {
        let bytecode = Bytes::from_static(&[]);
        let jump_table = JumpTable(Arc::new(bitvec![u8, Lsb0; 0; 0]));
        let _ = LegacyAnalyzedBytecode::new(bytecode, 0, jump_table);
    }
}
