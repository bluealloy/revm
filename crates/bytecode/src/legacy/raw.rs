use super::{JumpTable, LegacyAnalyzedBytecode};
use crate::opcode;
use bitvec::{bitvec, order::Lsb0, vec::BitVec};
use core::ops::Deref;
use primitives::Bytes;
use std::{sync::Arc, vec::Vec};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LegacyRawBytecode(pub Bytes);

impl LegacyRawBytecode {
    pub fn analysis(&self) -> JumpTable {
        analyze_legacy(&self.0)
    }

    pub fn into_analyzed(self) -> LegacyAnalyzedBytecode {
        let jump_table = self.analysis();
        let len = self.0.len();
        let mut padded_bytecode = Vec::with_capacity(len + 33);
        padded_bytecode.extend_from_slice(&self.0);
        padded_bytecode.resize(len + 33, 0);
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

/// Analyze the bytecode to find the jumpdests
pub fn analyze_legacy(bytetecode: &[u8]) -> JumpTable {
    let mut jumps: BitVec<u8> = bitvec![u8, Lsb0; 0; bytetecode.len()];

    let range = bytetecode.as_ptr_range();
    let start = range.start;
    let mut iterator = start;
    let end = range.end;
    while iterator < end {
        let opcode = unsafe { *iterator };
        if opcode::JUMPDEST == opcode {
            // SAFETY: jumps are max length of the code
            unsafe { jumps.set_unchecked(iterator.offset_from(start) as usize, true) }
            iterator = unsafe { iterator.offset(1) };
        } else {
            let push_offset = opcode.wrapping_sub(opcode::PUSH1);
            if push_offset < 32 {
                // SAFETY: iterator access range is checked in the while loop
                iterator = unsafe { iterator.offset((push_offset + 2) as isize) };
            } else {
                // SAFETY: iterator access range is checked in the while loop
                iterator = unsafe { iterator.offset(1) };
            }
        }
    }

    JumpTable(Arc::new(jumps))
}
