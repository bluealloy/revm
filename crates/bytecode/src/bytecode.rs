//! Module that contains the bytecode enum with all variants supported by Ethereum mainnet.
//!
//! Those are:
//! - Legacy bytecode with jump table analysis. Found in [`LegacyAnalyzedBytecode`]
//! - EIP-7702 bytecode, introduces in Prague and contains address to delegated account.

use std::sync::Arc;

use crate::{
    eip7702::{Eip7702Bytecode, EIP7702_MAGIC_BYTES},
    BytecodeDecodeError, JumpTable, LegacyAnalyzedBytecode, LegacyRawBytecode,
};
use primitives::{alloy_primitives::Sealable, keccak256, Address, Bytes, B256, KECCAK_EMPTY};

/// Main bytecode structure with all variants.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Bytecode {
    /// EIP-7702 delegated bytecode
    Eip7702(Arc<Eip7702Bytecode>),
    /// The bytecode has been analyzed for valid jump destinations.
    LegacyAnalyzed(Arc<LegacyAnalyzedBytecode>),
}

impl Default for Bytecode {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Sealable for Bytecode {
    #[inline]
    fn hash_slow(&self) -> B256 {
        self.hash_slow()
    }
}

impl Bytecode {
    /// Creates a new legacy analyzed [`Bytecode`] with exactly one STOP opcode.
    #[inline]
    pub fn new() -> Self {
        Self::LegacyAnalyzed(Arc::new(LegacyAnalyzedBytecode::default()))
    }

    /// Returns jump table if bytecode is analyzed.
    #[inline]
    pub fn legacy_jump_table(&self) -> Option<&JumpTable> {
        match &self {
            Self::LegacyAnalyzed(analyzed) => Some(analyzed.jump_table()),
            _ => None,
        }
    }

    /// Calculates hash of the bytecode.
    #[inline]
    pub fn hash_slow(&self) -> B256 {
        if self.is_empty() {
            KECCAK_EMPTY
        } else {
            keccak256(self.original_byte_slice())
        }
    }

    /// Returns `true` if bytecode is EIP-7702.
    #[inline]
    pub const fn is_eip7702(&self) -> bool {
        matches!(self, Self::Eip7702(_))
    }

    /// Creates a new legacy [`Bytecode`].
    #[inline]
    pub fn new_legacy(raw: Bytes) -> Self {
        Self::LegacyAnalyzed(Arc::new(LegacyRawBytecode(raw).into_analyzed()))
    }

    /// Creates a new raw [`Bytecode`].
    ///
    /// # Panics
    ///
    /// Panics if bytecode is in incorrect format. If you want to handle errors use [`Self::new_raw_checked`].
    #[inline]
    pub fn new_raw(bytecode: Bytes) -> Self {
        Self::new_raw_checked(bytecode).expect("Expect correct bytecode")
    }

    /// Creates a new EIP-7702 [`Bytecode`] from [`Address`].
    #[inline]
    pub fn new_eip7702(address: Address) -> Self {
        Self::Eip7702(Arc::new(Eip7702Bytecode::new(address)))
    }

    /// Creates a new raw [`Bytecode`].
    ///
    /// Returns an error on incorrect bytecode format.
    #[inline]
    pub fn new_raw_checked(bytes: Bytes) -> Result<Self, BytecodeDecodeError> {
        let prefix = bytes.get(..2);
        match prefix {
            Some(prefix) if prefix == &EIP7702_MAGIC_BYTES => {
                let eip7702 = Eip7702Bytecode::new_raw(bytes)?;
                Ok(Self::Eip7702(Arc::new(eip7702)))
            }
            _ => Ok(Self::new_legacy(bytes)),
        }
    }

    /// Create new checked bytecode.
    ///
    /// # Panics
    ///
    /// For possible panics see [`LegacyAnalyzedBytecode::new`].
    #[inline]
    pub fn new_analyzed(bytecode: Bytes, original_len: usize, jump_table: JumpTable) -> Self {
        Self::LegacyAnalyzed(Arc::new(LegacyAnalyzedBytecode::new(
            bytecode,
            original_len,
            jump_table,
        )))
    }

    /// Returns a reference to the bytecode.
    #[inline]
    pub fn bytecode(&self) -> &Bytes {
        match self {
            Self::LegacyAnalyzed(analyzed) => analyzed.bytecode(),
            Self::Eip7702(code) => code.raw(),
        }
    }

    /// Pointer to the executable bytecode.
    #[inline]
    pub fn bytecode_ptr(&self) -> *const u8 {
        self.bytecode().as_ptr()
    }

    /// Returns bytes.
    #[inline]
    pub fn bytes(&self) -> Bytes {
        self.bytes_ref().clone()
    }

    /// Returns raw bytes reference.
    #[inline]
    pub fn bytes_ref(&self) -> &Bytes {
        match self {
            Self::LegacyAnalyzed(analyzed) => analyzed.bytecode(),
            Self::Eip7702(code) => code.raw(),
        }
    }

    /// Returns raw bytes slice.
    #[inline]
    pub fn bytes_slice(&self) -> &[u8] {
        self.bytes_ref()
    }

    /// Returns the original bytecode.
    #[inline]
    pub fn original_bytes(&self) -> Bytes {
        match self {
            Self::LegacyAnalyzed(analyzed) => analyzed.original_bytes(),
            Self::Eip7702(eip7702) => eip7702.raw().clone(),
        }
    }

    /// Returns the original bytecode as a byte slice.
    #[inline]
    pub fn original_byte_slice(&self) -> &[u8] {
        match self {
            Self::LegacyAnalyzed(analyzed) => analyzed.original_byte_slice(),
            Self::Eip7702(eip7702) => eip7702.raw(),
        }
    }

    /// Returns the length of the original bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.original_byte_slice().len()
    }

    /// Returns whether the bytecode is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns an iterator over the opcodes in this bytecode, skipping immediates.
    /// This is useful if you want to ignore immediates and just see what opcodes are inside.
    #[inline]
    pub fn iter_opcodes(&self) -> crate::BytecodeIterator<'_> {
        crate::BytecodeIterator::new(self)
    }
}
