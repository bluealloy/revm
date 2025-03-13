use bitvec::vec::BitVec;
use primitives::hex;
use std::{fmt::Debug, sync::Arc};

/// A table of valid `jump` destinations. Cheap to clone and memory efficient, one bit per opcode.
#[derive(Clone, Default, PartialEq, Eq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JumpTable(pub Arc<BitVec<u8>>);

impl Debug for JumpTable {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("JumpTable")
            .field("map", &hex::encode(self.0.as_raw_slice()))
            .finish()
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
    /// Lenght represents number of bits inside slice.
    ///
    /// # Panics
    ///
    /// Panics if number of bits in slice is less than length.
    #[inline]
    pub fn from_slice(slice: &[u8], len: usize) -> Self {
        assert!(
            slice.len() * 8 >= len,
            "slice bit length {} is less than len {}",
            slice.len() * 8,
            len
        );
        let mut bitvec = BitVec::from_slice(slice);
        unsafe { bitvec.set_len(len) };
        Self(Arc::new(bitvec))
    }

    /// Checks if `pc` is a valid jump destination.
    #[inline]
    pub fn is_valid(&self, pc: usize) -> bool {
        pc < self.0.len() && unsafe { *self.0.get_unchecked(pc) }
    }
}
