use super::{analyze_legacy, LegacyAnalyzedBytecode};
use core::ops::Deref;
use primitives::Bytes;
use std::vec::Vec;

/// Used only as intermediate representation for legacy bytecode.
/// Please check [`LegacyAnalyzedBytecode`] for the main structure that is used in Revm.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LegacyRawBytecode(pub Bytes);

impl LegacyRawBytecode {
    /// Converts the raw bytecode into an analyzed bytecode.
    ///
    /// It extends the bytecode with 33 zero bytes and analyzes it to find the jumpdests.
    pub fn into_analyzed(self) -> LegacyAnalyzedBytecode {
        let len = self.0.len();
        let mut padded_bytecode = Vec::with_capacity(len + 33);
        padded_bytecode.extend_from_slice(&self.0);
        padded_bytecode.resize(len + 33, 0);
        let jump_table = analyze_legacy(&padded_bytecode);
        LegacyAnalyzedBytecode::new(padded_bytecode.into(), len, jump_table)
    }
}

impl From<Bytes> for LegacyRawBytecode {
    fn from(bytes: Bytes) -> Self {
        Self(bytes)
    }
}

impl<const N: usize> From<[u8; N]> for LegacyRawBytecode {
    fn from(bytes: [u8; N]) -> Self {
        Self(bytes.into())
    }
}

impl Deref for LegacyRawBytecode {
    type Target = Bytes;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
