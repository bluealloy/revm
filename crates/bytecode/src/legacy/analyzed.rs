use super::JumpTable;
use bitvec::{bitvec, order::Lsb0};
use primitives::Bytes;
use std::sync::Arc;

// Legacy analyzed
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LegacyAnalyzedBytecode {
    /// Bytecode with 32 zero bytes padding.
    bytecode: Bytes,
    /// Original bytes length.
    original_len: usize,
    /// Jump table.
    jump_table: JumpTable,
}

impl Default for LegacyAnalyzedBytecode {
    #[inline]
    fn default() -> Self {
        Self {
            bytecode: Bytes::from_static(&[0]),
            original_len: 0,
            jump_table: JumpTable(Arc::new(bitvec![u8, Lsb0; 0])),
        }
    }
}

impl LegacyAnalyzedBytecode {
    /// Create new analyzed bytecode.
    pub fn new(bytecode: Bytes, original_len: usize, jump_table: JumpTable) -> Self {
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

    /// Original bytes length.
    pub fn original_len(&self) -> usize {
        self.original_len
    }

    /// Original bytes without padding.
    pub fn original_bytes(&self) -> Bytes {
        self.bytecode.slice(..self.original_len)
    }

    /// Original bytes without padding.
    pub fn original_byte_slice(&self) -> &[u8] {
        &self.bytecode[..self.original_len]
    }

    /// Jumptable of analyzed bytes.
    pub fn jump_table(&self) -> &JumpTable {
        &self.jump_table
    }
}
