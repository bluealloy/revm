use bitvec::vec::BitVec;
use once_cell::race::OnceBox;
use primitives::hex;
use std::{fmt::Debug, sync::Arc};

/// A table of valid `jump` destinations. Cheap to clone and memory efficient, one bit per opcode.
#[derive(Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JumpTable(pub Arc<BitVec<u8>>);

impl Debug for JumpTable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("JumpTable")
            .field("map", &hex::encode(self.0.as_raw_slice()))
            .finish()
    }
}

impl Default for JumpTable {
    #[inline]
    fn default() -> Self {
        static DEFAULT: OnceBox<JumpTable> = OnceBox::new();
        DEFAULT.get_or_init(|| Self(Arc::default()).into()).clone()
    }
}

impl JumpTable {
    /// Gets the raw bytes of the jump map.
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        self.0.as_raw_slice()
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
        assert!(
            slice.len() * 8 >= bit_len,
            "slice bit length {} is less than bit_len {}",
            slice.len() * 8,
            bit_len
        );
        let mut bitvec = BitVec::from_slice(slice);
        unsafe { bitvec.set_len(bit_len) };
        Self(Arc::new(bitvec))
    }

    /// Checks if `pc` is a valid jump destination.
    #[inline]
    pub fn is_valid(&self, pc: usize) -> bool {
        pc < self.0.len() && unsafe { *self.0.get_unchecked(pc) }
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
        let jumptable = JumpTable::from_slice(slice, 3);
        assert_eq!(jumptable.0.len(), 3);
    }
}
