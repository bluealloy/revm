use crate::opcode;
use crate::primitives::{
    bitvec::prelude::{bitvec, BitVec, Lsb0},
    keccak256, Bytecode, BytecodeState, Bytes, JumpMap, B256, KECCAK_EMPTY,
};
use alloc::sync::Arc;
use core::fmt;

/// Perform bytecode analysis.
///
/// The analysis finds and caches valid jump destinations for later execution as an optimization step.
///
/// If the bytecode is already analyzed, it is returned as-is.
pub fn to_analysed(bytecode: Bytecode) -> Bytecode {
    let (bytecode, len) = match bytecode.state {
        BytecodeState::Raw => {
            let len = bytecode.bytecode.len();
            let checked = bytecode.to_checked();
            (checked.bytecode, len)
        }
        BytecodeState::Checked { len } => (bytecode.bytecode, len),
        _ => return bytecode,
    };
    let jump_map = analyze(bytecode.as_ref());

    Bytecode {
        bytecode,
        state: BytecodeState::Analysed { len, jump_map },
    }
}

/// Analyze bytecode to build a jump map.
fn analyze(code: &[u8]) -> JumpMap {
    let mut jumps: BitVec<u8> = bitvec![u8, Lsb0; 0; code.len()];

    let range = code.as_ptr_range();
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

    JumpMap(Arc::new(jumps))
}

#[derive(Clone)]
pub struct BytecodeLocked {
    bytecode: Bytes,
    len: usize,
    jump_map: JumpMap,
}

impl fmt::Debug for BytecodeLocked {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BytecodeLocked")
            .field("bytecode", &self.bytecode)
            .field("len", &self.len)
            .field(
                "jump_map",
                &crate::primitives::hex::encode(self.jump_map.as_slice()),
            )
            .finish()
    }
}

impl Default for BytecodeLocked {
    #[inline]
    fn default() -> Self {
        Bytecode::default()
            .try_into()
            .expect("Bytecode default is analysed code")
    }
}

impl TryFrom<Bytecode> for BytecodeLocked {
    type Error = ();

    #[inline]
    fn try_from(bytecode: Bytecode) -> Result<Self, Self::Error> {
        if let BytecodeState::Analysed { len, jump_map } = bytecode.state {
            Ok(BytecodeLocked {
                bytecode: bytecode.bytecode,
                len,
                jump_map,
            })
        } else {
            Err(())
        }
    }
}

impl BytecodeLocked {
    /// Returns a raw pointer to the underlying byte slice.
    #[inline]
    pub fn as_ptr(&self) -> *const u8 {
        self.bytecode.as_ptr()
    }

    /// Returns the length of the bytecode.
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns whether the bytecode is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Calculate hash of the bytecode.
    #[inline]
    pub fn hash_slow(&self) -> B256 {
        if self.is_empty() {
            KECCAK_EMPTY
        } else {
            keccak256(self.original_bytecode_slice())
        }
    }

    #[inline]
    pub fn unlock(self) -> Bytecode {
        Bytecode {
            bytecode: self.bytecode,
            state: BytecodeState::Analysed {
                len: self.len,
                jump_map: self.jump_map,
            },
        }
    }

    /// Returns the bytecode as a byte slice.
    #[inline]
    pub fn bytecode(&self) -> &[u8] {
        &self.bytecode
    }

    /// Returns the original bytecode as a byte slice.
    #[inline]
    pub fn original_bytecode_slice(&self) -> &[u8] {
        match self.bytecode.get(..self.len) {
            Some(slice) => slice,
            None => debug_unreachable!(
                "original_bytecode_slice OOB: {} > {}",
                self.len,
                self.bytecode.len()
            ),
        }
    }

    /// Returns a reference to the jump map.
    #[inline]
    pub fn jump_map(&self) -> &JumpMap {
        &self.jump_map
    }
}
