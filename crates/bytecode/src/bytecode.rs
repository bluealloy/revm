use crate::{
    eip7702::{Eip7702Bytecode, EIP7702_MAGIC_BYTES},
    BytecodeDecodeError, Eof, JumpTable, LegacyAnalyzedBytecode, LegacyRawBytecode,
    EOF_MAGIC_BYTES,
};
use core::fmt::Debug;
use primitives::{keccak256, Address, Bytes, B256, KECCAK_EMPTY};
use std::sync::Arc;

/// State of the [`Bytecode`] analysis
#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Bytecode {
    /// The bytecode has been analyzed for valid jump destinations.
    LegacyAnalyzed(LegacyAnalyzedBytecode),
    /// Ethereum Object Format
    Eof(Arc<Eof>),
    /// EIP-7702 delegated bytecode
    Eip7702(Eip7702Bytecode),
}

impl Default for Bytecode {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Bytecode {
    /// Creates a new legacy analyzed [`Bytecode`] with exactly one STOP opcode.
    #[inline]
    pub fn new() -> Self {
        Self::LegacyAnalyzed(LegacyAnalyzedBytecode::default())
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
    pub fn hash_slow(&self) -> B256 {
        if self.is_empty() {
            KECCAK_EMPTY
        } else {
            keccak256(self.original_byte_slice())
        }
    }

    /// Returns reference to the EOF if bytecode is EOF.
    #[inline]
    pub const fn eof(&self) -> Option<&Arc<Eof>> {
        match self {
            Self::Eof(eof) => Some(eof),
            _ => None,
        }
    }

    /// Returns `true` if bytecode is EOF.
    #[inline]
    pub const fn is_eof(&self) -> bool {
        matches!(self, Self::Eof(_))
    }

    /// Returns `true` if bytecode is EIP-7702.
    pub const fn is_eip7702(&self) -> bool {
        matches!(self, Self::Eip7702(_))
    }

    /// Creates a new legacy [`Bytecode`].
    #[inline]
    pub fn new_legacy(raw: Bytes) -> Self {
        Self::LegacyAnalyzed(LegacyRawBytecode(raw).into_analyzed())
    }

    /// Creates a new raw [`Bytecode`].
    ///
    /// # Panics
    ///
    /// Panics if bytecode is in incorrect format.
    #[inline]
    pub fn new_raw(bytecode: Bytes) -> Self {
        Self::new_raw_checked(bytecode).expect("Expect correct EOF bytecode")
    }

    /// Creates a new EIP-7702 [`Bytecode`] from [`Address`].
    #[inline]
    pub fn new_eip7702(address: Address) -> Self {
        Self::Eip7702(Eip7702Bytecode::new(address))
    }

    /// Creates a new raw [`Bytecode`].
    ///
    /// Returns an error on incorrect bytecode format.
    #[inline]
    pub fn new_raw_checked(bytes: Bytes) -> Result<Self, BytecodeDecodeError> {
        let prefix = bytes.get(..2);
        match prefix {
            Some(prefix) if prefix == &EOF_MAGIC_BYTES => {
                let eof = Eof::decode(bytes)?;
                Ok(Self::Eof(Arc::new(eof)))
            }
            Some(prefix) if prefix == &EIP7702_MAGIC_BYTES => {
                let eip7702 = Eip7702Bytecode::new_raw(bytes)?;
                Ok(Self::Eip7702(eip7702))
            }
            _ => Ok(Self::new_legacy(bytes)),
        }
    }

    /// Create new checked bytecode.
    ///
    /// # Safety
    ///
    /// Bytecode needs to end with `STOP` (`0x00`) opcode as checked bytecode assumes
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
    ///
    /// In case of EOF this will be the all code sections.
    #[inline]
    pub fn bytecode(&self) -> &Bytes {
        match self {
            Self::LegacyAnalyzed(analyzed) => analyzed.bytecode(),
            Self::Eof(eof) => &eof.body.code,
            Self::Eip7702(code) => code.raw(),
        }
    }

    /// Returns bytes.
    #[inline]
    pub fn bytes(&self) -> Bytes {
        match self {
            Self::LegacyAnalyzed(analyzed) => analyzed.bytecode().clone(),
            _ => self.original_bytes(),
        }
    }

    /// Returns bytes slice.
    #[inline]
    pub fn bytes_slice(&self) -> &[u8] {
        match self {
            Self::LegacyAnalyzed(analyzed) => analyzed.bytecode(),
            _ => self.original_byte_slice(),
        }
    }

    /// Returns a reference to the original bytecode.
    #[inline]
    pub fn original_bytes(&self) -> Bytes {
        match self {
            Self::LegacyAnalyzed(analyzed) => analyzed.original_bytes(),
            Self::Eof(eof) => eof.raw().clone(),
            Self::Eip7702(eip7702) => eip7702.raw().clone(),
        }
    }

    /// Returns the original bytecode as a byte slice.
    #[inline]
    pub fn original_byte_slice(&self) -> &[u8] {
        match self {
            Self::LegacyAnalyzed(analyzed) => analyzed.original_byte_slice(),
            Self::Eof(eof) => eof.raw(),
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
}

#[cfg(test)]
mod tests {
    use super::{Bytecode, Eof};
    use std::sync::Arc;

    #[test]
    fn eof_arc_clone() {
        let eof = Arc::new(Eof::default());
        let bytecode = Bytecode::Eof(Arc::clone(&eof));

        // Cloning the Bytecode should not clone the underlying Eof
        let cloned_bytecode = bytecode.clone();
        if let Bytecode::Eof(original_arc) = bytecode {
            if let Bytecode::Eof(cloned_arc) = cloned_bytecode {
                assert!(Arc::ptr_eq(&original_arc, &cloned_arc));
            } else {
                panic!("Cloned bytecode is not Eof");
            }
        } else {
            panic!("Original bytecode is not Eof");
        }
    }
}
