//! Module that contains the bytecode enum with all variants supported by Ethereum mainnet.
//!
//! Those are:
//! - Legacy bytecode with jump table analysis. Found in [`LegacyAnalyzedBytecode`]
//! - EIP-7702 bytecode, introduces in Prague and contains address to delegated account.

use crate::{
    eip7702::{Eip7702Bytecode, EIP7702_MAGIC_BYTES},
    BytecodeDecodeError, JumpTable, LegacyAnalyzedBytecode, LegacyRawBytecode,
};
use primitives::{
    alloy_primitives::Sealable, keccak256, Address, Bytes, OnceLock, B256, KECCAK_EMPTY,
};
use std::sync::Arc;

/// Main bytecode structure with all variants.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Bytecode(Arc<BytecodeKind>);

/// Inner bytecode representation with all variants.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub(crate) enum BytecodeKind {
    /// EIP-7702 delegated bytecode
    Eip7702(Eip7702Bytecode),
    /// The bytecode has been analyzed for valid jump destinations.
    LegacyAnalyzed(LegacyAnalyzedBytecode),
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
        static DEFAULT_BYTECODE: OnceLock<Bytecode> = OnceLock::new();
        DEFAULT_BYTECODE
            .get_or_init(|| Self::new_legacy_analyzed(LegacyAnalyzedBytecode::default()))
            .clone()
    }

    /// Creates a new legacy [`Bytecode`].
    #[inline]
    pub fn new_legacy(raw: Bytes) -> Self {
        Self::new_legacy_analyzed(LegacyRawBytecode(raw).into_analyzed())
    }

    /// Creates a new legacy analyzed [`Bytecode`] from a [`LegacyAnalyzedBytecode`].
    #[inline]
    pub fn new_legacy_analyzed(analyzed: LegacyAnalyzedBytecode) -> Self {
        Self::new_inner(BytecodeKind::LegacyAnalyzed(analyzed))
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
        Self::new_inner(BytecodeKind::Eip7702(Eip7702Bytecode::new(address)))
    }

    /// Creates a new raw [`Bytecode`].
    ///
    /// Returns an error on incorrect bytecode format.
    #[inline]
    pub fn new_raw_checked(bytes: Bytes) -> Result<Self, BytecodeDecodeError> {
        if bytes.starts_with(EIP7702_MAGIC_BYTES) {
            let eip7702 = Eip7702Bytecode::new_raw(bytes)?;
            Ok(Self::new_inner(BytecodeKind::Eip7702(eip7702)))
        } else {
            Ok(Self::new_legacy(bytes))
        }
    }

    /// Create new checked bytecode.
    ///
    /// # Panics
    ///
    /// For possible panics see [`LegacyAnalyzedBytecode::new`].
    #[inline]
    pub fn new_analyzed(bytecode: Bytes, original_len: usize, jump_table: JumpTable) -> Self {
        Self::new_legacy_analyzed(LegacyAnalyzedBytecode::new(
            bytecode,
            original_len,
            jump_table,
        ))
    }

    #[inline]
    fn new_inner(inner: BytecodeKind) -> Self {
        Self(inner.into())
    }

    #[inline]
    pub(crate) fn inner(&self) -> &BytecodeKind {
        &self.0
    }

    /// Returns `true` if bytecode is EIP-7702.
    #[inline]
    pub fn is_legacy(&self) -> bool {
        matches!(self.inner(), BytecodeKind::LegacyAnalyzed(_))
    }

    /// Returns the inner [`LegacyBytecode`] if this is an EIP-7702 bytecode.
    #[inline]
    pub fn legacy(&self) -> Option<&LegacyAnalyzedBytecode> {
        match self.inner() {
            BytecodeKind::LegacyAnalyzed(eip7702) => Some(eip7702),
            _ => None,
        }
    }

    /// Returns `true` if bytecode is EIP-7702.
    #[inline]
    pub fn is_eip7702(&self) -> bool {
        matches!(self.inner(), BytecodeKind::Eip7702(_))
    }

    /// Returns the inner [`Eip7702Bytecode`] if this is an EIP-7702 bytecode.
    #[inline]
    pub fn eip7702(&self) -> Option<&Eip7702Bytecode> {
        match self.inner() {
            BytecodeKind::Eip7702(eip7702) => Some(eip7702),
            _ => None,
        }
    }

    /// Returns jump table if bytecode is analyzed.
    #[inline]
    pub fn legacy_jump_table(&self) -> Option<&JumpTable> {
        Some(self.legacy()?.jump_table())
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

    /// Returns a reference to the bytecode.
    #[inline]
    pub fn bytecode(&self) -> &Bytes {
        match self.inner() {
            BytecodeKind::LegacyAnalyzed(analyzed) => analyzed.bytecode(),
            BytecodeKind::Eip7702(code) => code.raw(),
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
        self.bytecode()
    }

    /// Returns raw bytes slice.
    #[inline]
    pub fn bytes_slice(&self) -> &[u8] {
        self.bytes_ref()
    }

    /// Returns the original bytecode.
    #[inline]
    pub fn original_bytes(&self) -> Bytes {
        match self.inner() {
            BytecodeKind::LegacyAnalyzed(analyzed) => analyzed.original_bytes(),
            BytecodeKind::Eip7702(eip7702) => eip7702.raw().clone(),
        }
    }

    /// Returns the original bytecode as a byte slice.
    #[inline]
    pub fn original_byte_slice(&self) -> &[u8] {
        match self.inner() {
            BytecodeKind::LegacyAnalyzed(analyzed) => analyzed.original_byte_slice(),
            BytecodeKind::Eip7702(eip7702) => eip7702.raw(),
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
