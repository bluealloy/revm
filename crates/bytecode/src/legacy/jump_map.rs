use bitvec::vec::BitVec;
use core::fmt;
use primitives::hex;

/// A table of valid `jump` destinations.
///
/// It is immutable and memory efficient, with one bit per byte in the bytecode.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct JumpTable {
    table: Box<[u8]>,
    bit_len: usize,
}

// SAFETY: JumpTable is immutable, and just a simple Box<[u8]>, but len.
unsafe impl Send for JumpTable {}
unsafe impl Sync for JumpTable {}

impl fmt::Debug for JumpTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("JumpTable")
            .field("map", &hex::encode(self.as_slice()))
            .finish()
    }
}

impl Default for JumpTable {
    #[inline]
    fn default() -> Self {
        Self::new(Default::default())
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for JumpTable {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut bitvec = BitVec::<u8>::from_vec(self.table.to_vec());
        bitvec.resize(self.bit_len, false);
        bitvec.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for JumpTable {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        BitVec::deserialize(deserializer).map(Self::new)
    }
}

impl JumpTable {
    /// Create new JumpTable directly from an existing BitVec.
    #[inline]
    pub fn new(jumps: BitVec<u8>) -> Self {
        let bit_len = jumps.len();
        Self {
            table: jumps.into_vec().into_boxed_slice(),
            bit_len,
        }
    }

    /// Constructs a jump map from raw bytes and length.
    ///
    /// Bit length represents number of used bits inside slice.
    ///
    /// # Panics
    ///
    /// Panics if number of bits in slice is less than bit_len.
    #[inline]
    pub fn from_slice(slice: &[u8], bit_len: usize) -> Self {
        const BYTE_LEN: usize = 8;
        assert!(
            slice.len() * BYTE_LEN >= bit_len,
            "slice bit length {} is less than bit_len {}",
            slice.len() * BYTE_LEN,
            bit_len
        );

        Self {
            table: slice.into(),
            bit_len,
        }
    }

    /// Gets the raw bytes of the jump map.
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.table
    }

    /// Gets the bit length of the jump map.
    #[inline]
    pub fn len(&self) -> usize {
        self.bit_len
    }

    /// Returns true if the jump map is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Checks if `pc` is a valid jump destination.
    #[inline]
    pub fn is_valid(&self, pc: usize) -> bool {
        pc < self.bit_len && unsafe { *self.table.as_ptr().add(pc >> 3) & (1 << (pc & 7)) != 0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "slice bit length 8 is less than bit_len 10")]
    fn test_jump_table_from_slice_panic() {
        let slice = &[0x00];
        let _ = JumpTable::from_slice(slice, 10);
    }

    #[test]
    fn test_jump_table_from_slice() {
        let slice = &[0x00];
        let jump_table = JumpTable::from_slice(slice, 3);
        assert_eq!(jump_table.len(), 3);
    }

    #[test]
    fn test_is_valid() {
        let jump_table = JumpTable::from_slice(&[0x0D, 0x06], 13);

        assert_eq!(jump_table.len(), 13);

        assert!(jump_table.is_valid(0)); // valid
        assert!(!jump_table.is_valid(1));
        assert!(jump_table.is_valid(2)); // valid
        assert!(jump_table.is_valid(3)); // valid
        assert!(!jump_table.is_valid(4));
        assert!(!jump_table.is_valid(5));
        assert!(!jump_table.is_valid(6));
        assert!(!jump_table.is_valid(7));
        assert!(!jump_table.is_valid(8));
        assert!(jump_table.is_valid(9)); // valid
        assert!(jump_table.is_valid(10)); // valid
        assert!(!jump_table.is_valid(11));
        assert!(!jump_table.is_valid(12));
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serde_legacy_format() {
        let legacy_format = r#"
        {
            "order": "bitvec::order::Lsb0",
            "head": {
                "width": 8,
                "index": 0
            },
            "bits": 4,
            "data": [5]
        }"#;

        let table: JumpTable = serde_json::from_str(legacy_format).expect("Failed to deserialize");
        assert_eq!(table.len(), 4);
        assert!(table.is_valid(0));
        assert!(!table.is_valid(1));
        assert!(table.is_valid(2));
        assert!(!table.is_valid(3));
    }

    #[test]
    #[cfg(feature = "serde")]
    fn test_serde_roundtrip() {
        let original = JumpTable::from_slice(&[0x0D, 0x06], 13);

        // Serialize to JSON
        let serialized = serde_json::to_string(&original).expect("Failed to serialize");

        // Deserialize from JSON
        let deserialized: JumpTable =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        // Check that the deserialized table matches the original
        assert_eq!(original.len(), deserialized.len());
        assert_eq!(original.table, deserialized.table);
        assert_eq!(original, deserialized);

        // Verify functionality is preserved
        for i in 0..13 {
            assert_eq!(
                original.is_valid(i),
                deserialized.is_valid(i),
                "Mismatch at index {i}"
            );
        }
    }
}
