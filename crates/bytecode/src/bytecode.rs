//! Module that contains the bytecode enum with all variants supported by Ethereum mainnet.
//!
//! Those are:
//! - Legacy bytecode with jump table analysis. Found in [`LegacyAnalyzedBytecode`]
//! - EIP-7702 bytecode, introduces in Prague and contains address to delegated account.

use crate::{
    eip7702::{Eip7702Bytecode, EIP7702_MAGIC_BYTES},
    BytecodeDecodeError, JumpTable, LegacyAnalyzedBytecode, LegacyRawBytecode,
};
use core::fmt::Debug;
use primitives::{keccak256, Address, Bytes, B256, KECCAK_EMPTY};

/// Main bytecode structure with all variants.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Bytecode {
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
        Self::LegacyAnalyzed(LegacyRawBytecode(raw).into_analyzed())
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
        Self::Eip7702(Eip7702Bytecode::new(address))
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
                Ok(Self::Eip7702(eip7702))
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
        Self::LegacyAnalyzed(LegacyAnalyzedBytecode::new(
            bytecode,
            original_len,
            jump_table,
        ))
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

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::bytes;

    #[test]
    fn test_default() {
        let bytecode = Bytecode::default();
        assert!(matches!(bytecode, Bytecode::LegacyAnalyzed(_)));
        assert_eq!(bytecode.len(), 0); // Default is empty
    }

    #[test]
    fn test_new() {
        let bytecode = Bytecode::new();
        assert!(matches!(bytecode, Bytecode::LegacyAnalyzed(_)));
        assert_eq!(bytecode.len(), 0); // Default is empty
    }

    #[test]
    fn test_new_legacy() {
        let raw = bytes!("6060604052");
        let bytecode = Bytecode::new_legacy(raw.clone());
        assert!(matches!(bytecode, Bytecode::LegacyAnalyzed(_)));
        assert_eq!(bytecode.original_bytes(), raw);
    }

    #[test]
    fn test_new_eip7702() {
        let address = Address::new([0x11; 20]);
        let bytecode = Bytecode::new_eip7702(address);
        assert!(bytecode.is_eip7702());

        // Verify it matches the expected variant and extract the value
        assert!(
            matches!(&bytecode, Bytecode::Eip7702(eip7702) if eip7702.delegated_address == address)
        );
    }

    #[test]
    fn test_new_raw_legacy() {
        let raw = bytes!("6060604052");
        let bytecode = Bytecode::new_raw(raw.clone());
        assert!(matches!(bytecode, Bytecode::LegacyAnalyzed(_)));
        assert_eq!(bytecode.original_bytes(), raw);
    }

    #[test]
    fn test_new_raw_eip7702() {
        let address = Address::new([0x11; 20]);
        let eip7702_raw = Eip7702Bytecode::new(address).raw().clone();
        let bytecode = Bytecode::new_raw(eip7702_raw.clone());
        assert!(matches!(bytecode, Bytecode::Eip7702(_)));
        assert!(bytecode.is_eip7702());
    }

    #[test]
    #[should_panic(expected = "Expect correct bytecode")]
    fn test_new_raw_invalid_eip7702() {
        // Invalid EIP7702: correct magic but wrong length
        let invalid = bytes!("ef01");
        Bytecode::new_raw(invalid);
    }

    #[test]
    fn test_new_raw_checked_legacy() {
        let raw = bytes!("6060604052");
        let result = Bytecode::new_raw_checked(raw.clone());
        assert!(result.is_ok());
        let bytecode = result.unwrap();
        assert!(matches!(bytecode, Bytecode::LegacyAnalyzed(_)));
        assert_eq!(bytecode.original_bytes(), raw);
    }

    #[test]
    fn test_new_raw_checked_eip7702() {
        let address = Address::new([0x11; 20]);
        let eip7702_raw = Eip7702Bytecode::new(address).raw().clone();
        let result = Bytecode::new_raw_checked(eip7702_raw);
        assert!(result.is_ok());
        let bytecode = result.unwrap();
        assert!(matches!(bytecode, Bytecode::Eip7702(_)));
    }

    #[test]
    fn test_new_raw_checked_invalid_eip7702() {
        // Invalid EIP7702: correct magic but wrong length
        let invalid = bytes!("ef01");
        let result = Bytecode::new_raw_checked(invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_new_analyzed() {
        let raw = bytes!("00");
        // Create a BitVec with one bit for the STOP opcode
        let mut bitvec = bitvec::vec::BitVec::new();
        bitvec.push(false); // STOP is not a valid jump destination
        let jump_table = JumpTable::new(bitvec);
        let bytecode = Bytecode::new_analyzed(raw.clone(), 1, jump_table);
        assert!(matches!(bytecode, Bytecode::LegacyAnalyzed(_)));
        assert_eq!(bytecode.len(), 1);
    }

    #[test]
    fn test_legacy_jump_table() {
        // Test with legacy analyzed bytecode
        let bytecode = Bytecode::new();
        assert!(bytecode.legacy_jump_table().is_some());

        // Test with EIP7702 bytecode
        let address = Address::new([0x11; 20]);
        let eip7702 = Bytecode::new_eip7702(address);
        assert!(eip7702.legacy_jump_table().is_none());
    }

    #[test]
    fn test_hash_slow() {
        // Test empty bytecode
        let empty = Bytecode::new_legacy(Bytes::new());
        assert_eq!(empty.hash_slow(), KECCAK_EMPTY);

        // Test non-empty bytecode
        let bytecode = Bytecode::new_legacy(bytes!("6060604052"));
        let hash = bytecode.hash_slow();
        assert_ne!(hash, KECCAK_EMPTY);
        assert_eq!(hash, keccak256(bytecode.original_byte_slice()));
    }

    #[test]
    fn test_is_eip7702() {
        let legacy = Bytecode::new();
        assert!(!legacy.is_eip7702());

        let address = Address::new([0x11; 20]);
        let eip7702 = Bytecode::new_eip7702(address);
        assert!(eip7702.is_eip7702());
    }

    #[test]
    fn test_bytecode() {
        let raw = bytes!("6060604052");
        let legacy = Bytecode::new_legacy(raw.clone());
        // Legacy bytecode gets STOP appended during analysis
        let expected = bytes!("606060405200");
        assert_eq!(legacy.bytecode(), &expected);

        let address = Address::new([0x11; 20]);
        let eip7702 = Bytecode::new_eip7702(address);
        assert_eq!(eip7702.bytecode().len(), 23);
    }

    #[test]
    fn test_bytecode_ptr() {
        let bytecode = Bytecode::new();
        let ptr = bytecode.bytecode_ptr();
        assert!(!ptr.is_null());
        assert_eq!(ptr, bytecode.bytecode().as_ptr());
    }

    #[test]
    fn test_bytes() {
        let raw = bytes!("6060604052");
        let bytecode = Bytecode::new_legacy(raw.clone());
        // Legacy bytecode gets STOP appended during analysis
        let expected = bytes!("606060405200");
        assert_eq!(bytecode.bytes(), expected);
    }

    #[test]
    fn test_bytes_ref() {
        // Test legacy bytecode
        let raw = bytes!("6060604052");
        let bytecode = Bytecode::new_legacy(raw.clone());
        // Legacy bytecode gets STOP appended during analysis
        let expected = bytes!("606060405200");
        assert_eq!(bytecode.bytes_ref().as_ref(), expected.as_ref());

        // Test EIP7702 bytecode
        let address = Address::new([0x11; 20]);
        let eip7702 = Bytecode::new_eip7702(address);
        assert_eq!(eip7702.bytes_ref().len(), 23);
    }

    #[test]
    fn test_bytes_slice() {
        let raw = bytes!("6060604052");
        let bytecode = Bytecode::new_legacy(raw.clone());
        // Legacy bytecode gets STOP appended during analysis
        let expected = bytes!("606060405200");
        assert_eq!(bytecode.bytes_slice(), expected.as_ref());
    }

    #[test]
    fn test_original_bytes() {
        // Test legacy bytecode
        let raw = bytes!("6060604052");
        let legacy = Bytecode::new_legacy(raw.clone());
        assert_eq!(legacy.original_bytes(), raw);

        // Test EIP7702 bytecode
        let address = Address::new([0x11; 20]);
        let eip7702 = Bytecode::new_eip7702(address);
        assert_eq!(eip7702.original_bytes().len(), 23);
    }

    #[test]
    fn test_original_byte_slice() {
        let raw = bytes!("6060604052");
        let legacy = Bytecode::new_legacy(raw.clone());
        assert_eq!(legacy.original_byte_slice(), raw.as_ref());

        let address = Address::new([0x11; 20]);
        let eip7702 = Bytecode::new_eip7702(address);
        assert_eq!(eip7702.original_byte_slice().len(), 23);
    }

    #[test]
    fn test_len() {
        let empty = Bytecode::new_legacy(Bytes::new());
        assert_eq!(empty.len(), 0);

        let bytecode = Bytecode::new_legacy(bytes!("6060604052"));
        assert_eq!(bytecode.len(), 5);
    }

    #[test]
    fn test_is_empty() {
        let empty = Bytecode::new_legacy(Bytes::new());
        assert!(empty.is_empty());

        let non_empty = Bytecode::new_legacy(bytes!("60"));
        assert!(!non_empty.is_empty());
    }

    #[test]
    fn test_iter_opcodes() {
        let bytecode = Bytecode::new_legacy(bytes!("6001600201"));
        let opcodes: Vec<_> = bytecode.iter_opcodes().collect();
        assert_eq!(opcodes.len(), 4); // PUSH1, PUSH1, ADD, STOP (appended)
    }

    #[test]
    fn test_clone() {
        let bytecode = Bytecode::new_legacy(bytes!("6060604052"));
        let cloned = bytecode.clone();
        assert_eq!(bytecode, cloned);
    }

    #[test]
    fn test_debug() {
        let bytecode = Bytecode::new();
        let debug_str = format!("{:?}", bytecode);
        assert!(debug_str.contains("LegacyAnalyzed"));
    }

    #[test]
    fn test_eq() {
        let bytecode1 = Bytecode::new_legacy(bytes!("6060604052"));
        let bytecode2 = Bytecode::new_legacy(bytes!("6060604052"));
        let bytecode3 = Bytecode::new_legacy(bytes!("6001"));

        assert_eq!(bytecode1, bytecode2);
        assert_ne!(bytecode1, bytecode3);
    }

    #[test]
    fn test_hash_trait() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let bytecode1 = Bytecode::new_legacy(bytes!("6060604052"));
        let bytecode2 = Bytecode::new_legacy(bytes!("6060604052"));

        let mut hasher1 = DefaultHasher::new();
        bytecode1.hash(&mut hasher1);
        let hash1 = hasher1.finish();

        let mut hasher2 = DefaultHasher::new();
        bytecode2.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_ord() {
        let bytecode1 = Bytecode::new_legacy(bytes!("00"));
        let bytecode2 = Bytecode::new_legacy(bytes!("01"));

        assert!(bytecode1 < bytecode2);
        assert_eq!(bytecode1.cmp(&bytecode1), std::cmp::Ordering::Equal);
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde() {
        use serde::{Deserialize, Serialize};

        // Test that the type implements Serialize and Deserialize traits
        fn assert_serde<T: Serialize + for<'de> Deserialize<'de>>() {}
        assert_serde::<Bytecode>();
    }
}
