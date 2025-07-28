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
/// Legacy bytecode can be padded with up to 33 zero bytes at the end. This padding ensures that:
/// - the bytecode always ends with a valid STOP (0x00) opcode.
/// - there aren't incomplete immediates, meaning we can skip bounds checks in `PUSH*` instructions.
///
/// The non-padded length is stored in order to be able to copy the original bytecode.
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
    /// The potentially padded bytecode.
    bytecode: Bytes,
    /// The original bytecode length.
    original_len: usize,
    /// The jump table.
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
    /// Analyzes the bytecode.
    ///
    /// See [`LegacyAnalyzedBytecode`] for more details.
    pub fn analyze(bytecode: Bytes) -> Self {
        let original_len = bytecode.len();
        let (jump_table, padded_bytecode) = super::analysis::analyze_legacy(bytecode);
        Self::new(padded_bytecode, original_len, jump_table)
    }

    /// Creates new analyzed bytecode.
    ///
    /// Prefer instantiating using [`analyze`](Self::analyze) instead.
    ///
    /// # Panics
    ///
    /// * If `original_len` is greater than `bytecode.len()`
    /// * If jump table length is less than `original_len`.
    /// * If last bytecode byte is not `0x00` or if bytecode is empty.
    pub fn new(bytecode: Bytes, original_len: usize, jump_table: JumpTable) -> Self {
        assert!(
            original_len <= bytecode.len(),
            "original_len is greater than bytecode length"
        );
        assert!(
            original_len <= jump_table.len(),
            "jump table length is less than original length"
        );
        assert!(!bytecode.is_empty(), "bytecode cannot be empty");

        if let Some(&last_opcode) = bytecode.last() {
            assert!(
                opcode::OpCode::info_by_op(last_opcode)
                    .map(|o| o.is_terminating())
                    .unwrap_or(false),
                "last bytecode byte should be terminating"
            );
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
    #[should_panic(expected = "jump table length is less than original length")]
    fn test_panic_on_short_jump_table() {
        let bytecode = Bytes::from_static(&[opcode::PUSH1, 0x01]);
        let bytecode = LegacyRawBytecode(bytecode).into_analyzed();
        let jump_table = JumpTable::new(bitvec![u8, Lsb0; 0; 1]);
        let _ = LegacyAnalyzedBytecode::new(bytecode.bytecode, bytecode.original_len, jump_table);
    }

    #[test]
    #[should_panic(expected = "bytecode cannot be empty")]
    fn test_panic_on_empty_bytecode() {
        let bytecode = Bytes::from_static(&[]);
        let jump_table = JumpTable::new(bitvec![u8, Lsb0; 0; 0]);
        let _ = LegacyAnalyzedBytecode::new(bytecode, 0, jump_table);
    }
}
