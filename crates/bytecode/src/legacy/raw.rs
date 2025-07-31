use super::LegacyAnalyzedBytecode;
use core::ops::Deref;
use primitives::Bytes;

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

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::bytes;

    #[test]
    fn test_into_analyzed() {
        let raw_bytes = bytes!("6060604052");
        let raw = LegacyRawBytecode(raw_bytes.clone());
        let analyzed = raw.into_analyzed();

        // Verify it was analyzed
        assert_eq!(analyzed.original_len(), 5);
        // Analyzed bytecode should have STOP appended
        assert_eq!(analyzed.bytecode().len(), 6);
    }

    #[test]
    fn test_from_bytes() {
        let bytes = bytes!("6060604052");
        let raw = LegacyRawBytecode::from(bytes.clone());
        assert_eq!(raw.0, bytes);
    }

    #[test]
    fn test_from_array() {
        let array = [0x60, 0x60, 0x60, 0x40, 0x52];
        let raw = LegacyRawBytecode::from(array);
        assert_eq!(raw.0, bytes!("6060604052"));
    }

    #[test]
    fn test_deref() {
        let bytes = bytes!("6060604052");
        let raw = LegacyRawBytecode(bytes.clone());

        // Test that we can use Bytes methods via deref
        assert_eq!(raw.len(), 5);
        assert_eq!(raw.as_ref(), bytes.as_ref());
        assert!(!raw.is_empty());
    }

    #[test]
    fn test_clone() {
        let raw = LegacyRawBytecode(bytes!("6060604052"));
        let cloned = raw.clone();
        assert_eq!(raw, cloned);
    }

    #[test]
    fn test_debug() {
        let raw = LegacyRawBytecode(bytes!("6060"));
        let debug_str = format!("{raw:?}");
        assert!(debug_str.contains("LegacyRawBytecode"));
        assert!(debug_str.contains("6060"));
    }

    #[test]
    fn test_eq() {
        let raw1 = LegacyRawBytecode(bytes!("6060604052"));
        let raw2 = LegacyRawBytecode(bytes!("6060604052"));
        let raw3 = LegacyRawBytecode(bytes!("6001"));

        assert_eq!(raw1, raw2);
        assert_ne!(raw1, raw3);
    }

    #[test]
    fn test_hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let raw1 = LegacyRawBytecode(bytes!("6060604052"));
        let raw2 = LegacyRawBytecode(bytes!("6060604052"));

        let mut hasher1 = DefaultHasher::new();
        raw1.hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = DefaultHasher::new();
        raw2.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_ord() {
        let raw1 = LegacyRawBytecode(bytes!("00"));
        let raw2 = LegacyRawBytecode(bytes!("01"));

        assert!(raw1 < raw2);
        assert_eq!(raw1.cmp(&raw1), std::cmp::Ordering::Equal);
    }

    #[test]
    fn test_empty_bytecode() {
        let raw = LegacyRawBytecode(Bytes::new());
        let analyzed = raw.into_analyzed();

        // Even empty bytecode gets STOP appended
        assert_eq!(analyzed.original_len(), 0);
        assert_eq!(analyzed.bytecode().len(), 1);
        assert_eq!(analyzed.bytecode()[0], 0x00); // STOP opcode
    }

    #[test]
    fn test_from_small_array() {
        let array: [u8; 1] = [0x00];
        let raw = LegacyRawBytecode::from(array);
        assert_eq!(raw.0, bytes!("00"));
    }

    #[test]
    fn test_from_large_array() {
        let array: [u8; 32] = [0x60; 32];
        let raw = LegacyRawBytecode::from(array);
        assert_eq!(raw.0.len(), 32);
        assert!(raw.0.iter().all(|&b| b == 0x60));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde() {
        use serde::{Deserialize, Serialize};

        // Test that the type implements Serialize and Deserialize traits
        fn assert_serde<T: Serialize + for<'de> Deserialize<'de>>() {}
        assert_serde::<LegacyRawBytecode>();
    }
}
