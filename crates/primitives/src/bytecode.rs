use crate::{keccak256, B256, KECCAK_EMPTY};
use alloc::{sync::Arc, vec, vec::Vec};
use bitvec::prelude::{bitvec, Lsb0};
use bitvec::vec::BitVec;
use bytes::Bytes;

/// A map of valid `jump` destinations.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JumpMap(pub Arc<BitVec>);

impl JumpMap {
    /// Check if `pc` is a valid jump destination.
    pub fn is_valid(&self, pc: usize) -> bool {
        pc < self.0.len() && self.0[pc]
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BytecodeState {
    Raw,
    Checked { len: usize },
    Analysed { len: usize, jump_map: JumpMap },
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Bytecode {
    #[cfg_attr(feature = "serde", serde(with = "crate::utilities::serde_hex_bytes"))]
    pub bytecode: Bytes,
    pub hash: B256,
    pub state: BytecodeState,
}

impl Default for Bytecode {
    fn default() -> Self {
        Bytecode::new()
    }
}

impl Bytecode {
    /// Create [`Bytecode`] with one STOP opcode.
    pub fn new() -> Self {
        Bytecode {
            bytecode: vec![0].into(),
            hash: KECCAK_EMPTY,
            state: BytecodeState::Analysed {
                len: 0,
                jump_map: JumpMap(Arc::new(bitvec![0])),
            },
        }
    }

    pub fn new_raw(bytecode: Bytes) -> Self {
        let hash = if bytecode.is_empty() {
            KECCAK_EMPTY
        } else {
            keccak256(&bytecode)
        };
        Self {
            bytecode,
            hash,
            state: BytecodeState::Raw,
        }
    }

    /// Create new raw Bytecode with hash
    ///
    /// # Safety
    /// Hash need to be appropriate keccak256 over bytecode.
    pub unsafe fn new_raw_with_hash(bytecode: Bytes, hash: B256) -> Self {
        Self {
            bytecode,
            hash,
            state: BytecodeState::Raw,
        }
    }

    /// Create new checked bytecode
    ///
    /// # Safety
    /// Bytecode need to end with STOP (0x00) opcode as checked bytecode assumes
    /// that it is safe to iterate over bytecode without checking lengths
    pub unsafe fn new_checked(bytecode: Bytes, len: usize, hash: Option<B256>) -> Self {
        let hash = match hash {
            None if len == 0 => KECCAK_EMPTY,
            None => keccak256(&bytecode),
            Some(hash) => hash,
        };
        Self {
            bytecode,
            hash,
            state: BytecodeState::Checked { len },
        }
    }

    pub fn bytes(&self) -> &Bytes {
        &self.bytecode
    }

    pub fn original_bytes(&self) -> Bytes {
        match self.state {
            BytecodeState::Raw => self.bytecode.clone(),
            BytecodeState::Checked { len } | BytecodeState::Analysed { len, .. } => {
                self.bytecode.slice(0..len)
            }
        }
    }

    pub fn hash(&self) -> B256 {
        self.hash
    }

    pub fn state(&self) -> &BytecodeState {
        &self.state
    }

    pub fn is_empty(&self) -> bool {
        match self.state {
            BytecodeState::Raw => self.bytecode.is_empty(),
            BytecodeState::Checked { len } => len == 0,
            BytecodeState::Analysed { len, .. } => len == 0,
        }
    }

    pub fn len(&self) -> usize {
        match self.state {
            BytecodeState::Raw => self.bytecode.len(),
            BytecodeState::Checked { len, .. } => len,
            BytecodeState::Analysed { len, .. } => len,
        }
    }

    pub fn to_checked(self) -> Self {
        match self.state {
            BytecodeState::Raw => {
                let len = self.bytecode.len();
                let mut bytecode: Vec<u8> = Vec::from(self.bytecode.as_ref());
                bytecode.resize(len + 33, 0);
                Self {
                    bytecode: bytecode.into(),
                    hash: self.hash,
                    state: BytecodeState::Checked { len },
                }
            }
            _ => self,
        }
    }
}
