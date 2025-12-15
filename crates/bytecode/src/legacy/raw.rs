use super::LegacyAnalyzedBytecode;
use core::ops::Deref;
use primitives::Bytes;
use std::sync::Arc;

/// Used only as intermediate representation for legacy bytecode.
///
/// See [`LegacyAnalyzedBytecode`] for the main structure that is used in Revm.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LegacyRawBytecode(pub Bytes);

impl LegacyRawBytecode {
    /// Analyzes the bytecode, instantiating a [`LegacyAnalyzedBytecode`].
    pub fn into_analyzed(self) -> LegacyAnalyzedBytecode {
        LegacyAnalyzedBytecode::analyze(self.0)
    }

    /// Analyzes the bytecode, instantiating a [`LegacyAnalyzedBytecode`] and wrapping it in an [`Arc`].
    pub fn into_analyzed_arc(self) -> Arc<LegacyAnalyzedBytecode> {
        Arc::new(self.into_analyzed())
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
