//! Module that contains the bytecode struct with all variants supported by Ethereum mainnet.
//!
//! Those are:
//! - Legacy bytecode with jump table analysis
//! - EIP-7702 bytecode, introduced in Prague and contains address to delegated account

use crate::{
    eip7702::{Eip7702DecodeError, EIP7702_MAGIC_BYTES, EIP7702_VERSION},
    BytecodeDecodeError, JumpTable, LegacyRawBytecode,
};
use primitives::{
    alloy_primitives::Sealable, keccak256, Address, Bytes, OnceLock, B256, KECCAK_EMPTY,
};
use std::sync::Arc;

/// Main bytecode structure.
///
/// This is a wrapper around an `Arc<BytecodeInner>` that provides a convenient API.
#[derive(Clone, Debug)]
pub struct Bytecode(Arc<BytecodeInner>);

/// Inner bytecode representation.
///
/// This struct is flattened to avoid nested allocations. The `kind` field determines
/// how the bytecode should be interpreted.
#[derive(Debug)]
pub struct BytecodeInner {
    /// The kind of bytecode (Legacy or EIP-7702).
    kind: BytecodeKind,
    /// The bytecode bytes.
    ///
    /// For legacy bytecode, this may be padded with zeros at the end.
    /// For EIP-7702 bytecode, this is exactly 23 bytes.
    bytecode: Bytes,
    /// The original length of the bytecode before padding.
    ///
    /// For EIP-7702 bytecode, this is always 23.
    original_len: usize,
    /// The jump table for legacy bytecode. Empty for EIP-7702.
    jump_table: JumpTable,
    /// Cached hash of the original bytecode.
    hash: OnceLock<B256>,
}

/// The kind of bytecode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BytecodeKind {
    /// Legacy analyzed bytecode with jump table.
    #[default]
    LegacyAnalyzed,
    /// EIP-7702 delegated bytecode.
    Eip7702,
}

impl Default for Bytecode {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl PartialEq for Bytecode {
    fn eq(&self, other: &Self) -> bool {
        self.0.kind == other.0.kind && self.original_byte_slice() == other.original_byte_slice()
    }
}

impl Eq for Bytecode {}

impl core::hash::Hash for Bytecode {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.0.kind.hash(state);
        self.original_byte_slice().hash(state);
    }
}

impl PartialOrd for Bytecode {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Bytecode {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.0
            .kind
            .cmp(&other.0.kind)
            .then_with(|| self.original_byte_slice().cmp(&other.original_byte_slice()))
    }
}

impl Sealable for Bytecode {
    #[inline]
    fn hash_slow(&self) -> B256 {
        self.hash_slow()
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Bytecode {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("Bytecode", 4)?;
        s.serialize_field("kind", &self.0.kind)?;
        s.serialize_field("bytecode", &self.0.bytecode)?;
        s.serialize_field("original_len", &self.0.original_len)?;
        s.serialize_field("jump_table", &self.0.jump_table)?;
        s.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Bytecode {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(serde::Deserialize)]
        struct Inner {
            kind: BytecodeKind,
            bytecode: Bytes,
            original_len: usize,
            jump_table: JumpTable,
        }
        let inner = Inner::deserialize(deserializer)?;
        Ok(Self(Arc::new(BytecodeInner {
            kind: inner.kind,
            bytecode: inner.bytecode,
            original_len: inner.original_len,
            jump_table: inner.jump_table,
            hash: OnceLock::new(),
        })))
    }
}

impl Bytecode {
    /// Creates a new legacy analyzed [`Bytecode`] with exactly one STOP opcode.
    #[inline]
    pub fn new() -> Self {
        static DEFAULT_BYTECODE: OnceLock<Bytecode> = OnceLock::new();
        DEFAULT_BYTECODE
            .get_or_init(|| {
                Self(Arc::new(BytecodeInner {
                    kind: BytecodeKind::LegacyAnalyzed,
                    bytecode: Bytes::from_static(&[0]),
                    original_len: 0,
                    jump_table: JumpTable::default(),
                    hash: OnceLock::new(),
                }))
            })
            .clone()
    }

    /// Creates a new legacy [`Bytecode`] by analyzing raw bytes.
    #[inline]
    pub fn new_legacy(raw: Bytes) -> Self {
        let analyzed = LegacyRawBytecode(raw).into_analyzed();
        Self(Arc::new(BytecodeInner {
            kind: BytecodeKind::LegacyAnalyzed,
            original_len: analyzed.original_len(),
            bytecode: analyzed.bytecode().clone(),
            jump_table: analyzed.jump_table().clone(),
            hash: OnceLock::new(),
        }))
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
        let raw: Bytes = [EIP7702_MAGIC_BYTES, &[EIP7702_VERSION], &address[..]]
            .concat()
            .into();
        Self(Arc::new(BytecodeInner {
            kind: BytecodeKind::Eip7702,
            original_len: raw.len(),
            bytecode: raw,
            jump_table: JumpTable::default(),
            hash: OnceLock::new(),
        }))
    }

    /// Creates a new raw [`Bytecode`].
    ///
    /// Returns an error on incorrect bytecode format.
    #[inline]
    pub fn new_raw_checked(bytes: Bytes) -> Result<Self, BytecodeDecodeError> {
        if bytes.starts_with(EIP7702_MAGIC_BYTES) {
            Self::new_eip7702_raw(bytes).map_err(Into::into)
        } else {
            Ok(Self::new_legacy(bytes))
        }
    }

    /// Creates a new EIP-7702 [`Bytecode`] from raw bytes.
    ///
    /// Returns an error if the bytes are not valid EIP-7702 bytecode.
    #[inline]
    pub fn new_eip7702_raw(bytes: Bytes) -> Result<Self, Eip7702DecodeError> {
        if bytes.len() != 23 {
            return Err(Eip7702DecodeError::InvalidLength);
        }
        if !bytes.starts_with(EIP7702_MAGIC_BYTES) {
            return Err(Eip7702DecodeError::InvalidMagic);
        }
        if bytes[2] != EIP7702_VERSION {
            return Err(Eip7702DecodeError::UnsupportedVersion);
        }
        Ok(Self(Arc::new(BytecodeInner {
            kind: BytecodeKind::Eip7702,
            original_len: bytes.len(),
            bytecode: bytes,
            jump_table: JumpTable::default(),
            hash: OnceLock::new(),
        })))
    }

    /// Create new checked bytecode from pre-analyzed components.
    ///
    /// # Panics
    ///
    /// * If `original_len` is greater than `bytecode.len()`
    /// * If jump table length is less than `original_len`
    /// * If bytecode is empty
    #[inline]
    pub fn new_analyzed(bytecode: Bytes, original_len: usize, jump_table: JumpTable) -> Self {
        assert!(
            original_len <= bytecode.len(),
            "original_len is greater than bytecode length"
        );
        assert!(
            original_len <= jump_table.len(),
            "jump table length is less than original length"
        );
        assert!(!bytecode.is_empty(), "bytecode cannot be empty");
        Self(Arc::new(BytecodeInner {
            kind: BytecodeKind::LegacyAnalyzed,
            bytecode,
            original_len,
            jump_table,
            hash: OnceLock::new(),
        }))
    }

    /// Returns a reference to the inner bytecode.
    #[inline]
    pub fn inner(&self) -> &BytecodeInner {
        &self.0
    }

    /// Returns the kind of bytecode.
    #[inline]
    pub fn kind(&self) -> BytecodeKind {
        self.0.kind
    }

    /// Returns `true` if bytecode is legacy.
    #[inline]
    pub fn is_legacy(&self) -> bool {
        self.0.kind == BytecodeKind::LegacyAnalyzed
    }

    /// Returns `true` if bytecode is EIP-7702.
    #[inline]
    pub fn is_eip7702(&self) -> bool {
        self.0.kind == BytecodeKind::Eip7702
    }

    /// Returns the EIP-7702 delegated address if this is EIP-7702 bytecode.
    #[inline]
    pub fn eip7702_address(&self) -> Option<Address> {
        if self.is_eip7702() {
            Some(Address::from_slice(&self.0.bytecode[3..23]))
        } else {
            None
        }
    }

    /// Returns jump table if bytecode is legacy analyzed.
    #[inline]
    pub fn legacy_jump_table(&self) -> Option<&JumpTable> {
        if self.is_legacy() {
            Some(&self.0.jump_table)
        } else {
            None
        }
    }

    /// Calculates or returns cached hash of the bytecode.
    #[inline]
    pub fn hash_slow(&self) -> B256 {
        *self.0.hash.get_or_init(|| {
            if self.is_empty() {
                KECCAK_EMPTY
            } else {
                keccak256(self.original_byte_slice())
            }
        })
    }

    /// Returns a reference to the bytecode bytes.
    ///
    /// For legacy bytecode, this includes padding. For EIP-7702, this is the raw bytes.
    #[inline]
    pub fn bytecode(&self) -> &Bytes {
        &self.0.bytecode
    }

    /// Pointer to the bytecode bytes.
    #[inline]
    pub fn bytecode_ptr(&self) -> *const u8 {
        self.0.bytecode.as_ptr()
    }

    /// Returns a clone of the bytecode bytes.
    #[inline]
    pub fn bytes(&self) -> Bytes {
        self.0.bytecode.clone()
    }

    /// Returns a reference to the bytecode bytes.
    #[inline]
    pub fn bytes_ref(&self) -> &Bytes {
        &self.0.bytecode
    }

    /// Returns the bytecode as a slice.
    #[inline]
    pub fn bytes_slice(&self) -> &[u8] {
        &self.0.bytecode
    }

    /// Returns the original bytecode without padding.
    #[inline]
    pub fn original_bytes(&self) -> Bytes {
        self.0.bytecode.slice(..self.0.original_len)
    }

    /// Returns the original bytecode as a byte slice without padding.
    #[inline]
    pub fn original_byte_slice(&self) -> &[u8] {
        &self.0.bytecode[..self.0.original_len]
    }

    /// Returns the length of the original bytes (without padding).
    #[inline]
    pub fn len(&self) -> usize {
        self.0.original_len
    }

    /// Returns whether the bytecode is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0.original_len == 0
    }

    /// Returns an iterator over the opcodes in this bytecode, skipping immediates.
    #[inline]
    pub fn iter_opcodes(&self) -> crate::BytecodeIterator<'_> {
        crate::BytecodeIterator::new(self)
    }
}
