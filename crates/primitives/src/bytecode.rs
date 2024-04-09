pub mod eof;
pub mod legacy;

pub use eof::Eof;
pub use legacy::{JumpTable, LegacyAnalyzedBytecode};

use crate::{keccak256, Bytes, B256, KECCAK_EMPTY};

/// State of the [`Bytecode`] analysis.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Bytecode {
    /// No analysis has been performed.
    LegacyRaw(Bytes),
    /// The bytecode has been analyzed for valid jump destinations.
    LegacyAnalyzed(LegacyAnalyzedBytecode),
    /// Ethereum Object Format
    Eof(Eof),
}

impl Default for Bytecode {
    #[inline]
    fn default() -> Self {
        // Creates a new legacy analyzed [`Bytecode`] with exactly one STOP opcode.
        Self::new()
    }
}

impl Bytecode {
    // Creates a new legacy analyzed [`Bytecode`] with exactly one STOP opcode.
    #[inline]
    pub fn new() -> Self {
        Self::LegacyAnalyzed(LegacyAnalyzedBytecode::default())
    }

    /// Return jump table if bytecode is analyzed
    #[inline]
    pub fn legacy_jump_table(&self) -> Option<&JumpTable> {
        match &self {
            Self::LegacyAnalyzed(analyzed) => Some(analyzed.jump_table()),
            _ => None,
        }
    }

    /// Calculate hash of the bytecode.
    pub fn hash_slow(&self) -> B256 {
        if self.is_empty() {
            KECCAK_EMPTY
        } else {
            keccak256(&self.original_bytes())
        }
    }

    /// Return reference to the EOF if bytecode is EOF.
    pub fn eof(&self) -> Option<&Eof> {
        match self {
            Self::Eof(eof) => Some(eof),
            _ => None,
        }
    }

    /// Return true if bytecode is EOF.
    pub fn is_eof(&self) -> bool {
        matches!(self, Self::Eof(_))
    }

    /// Creates a new raw [`Bytecode`].
    #[inline]
    pub fn new_raw(bytecode: Bytes) -> Self {
        Self::LegacyRaw(bytecode)
    }

    /// Create new checked bytecode.
    ///
    /// # Safety
    ///
    /// Bytecode needs to end with STOP (0x00) opcode as checked bytecode assumes
    /// that it is safe to iterate over bytecode without checking lengths.
    pub unsafe fn new_analyzed(
        bytecode: Bytes,
        original_len: usize,
        jump_table: JumpTable,
    ) -> Self {
        Self::LegacyAnalyzed(LegacyAnalyzedBytecode::new(
            bytecode,
            original_len,
            jump_table,
        ))
    }

    /// Returns a reference to the bytecode.
    /// In case of EOF this will be the first code section.
    #[inline]
    pub fn bytecode_bytes(&self) -> Bytes {
        match self {
            Self::LegacyRaw(bytes) => bytes.clone(),
            Self::LegacyAnalyzed(analyzed) => analyzed.bytes(),
            Self::Eof(eof) => eof
                .body
                .code(0)
                .expect("Valid EOF has at least one code section")
                .clone(),
        }
    }

    /// Returns false if bytecode can't be executed in Interpreter.
    pub fn is_execution_ready(&self) -> bool {
        !matches!(self, Self::LegacyRaw(_))
    }

    /// Returns a reference to the original bytecode.
    #[inline]
    pub fn original_bytes(&self) -> Bytes {
        match self {
            Self::LegacyRaw(bytes) => bytes.clone(),
            Self::LegacyAnalyzed(analyzed) => analyzed.original_bytes(),
            Self::Eof(eof) => eof.raw().clone(),
        }
    }

    /// Returns the length of the raw bytes.
    #[inline]
    pub fn len(&self) -> usize {
        match self {
            Self::LegacyRaw(bytes) => bytes.len(),
            Self::LegacyAnalyzed(analyzed) => analyzed.original_len(),
            Self::Eof(eof) => eof.size(),
        }
    }

    /// Returns whether the bytecode is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
