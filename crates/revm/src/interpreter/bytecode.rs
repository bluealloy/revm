use super::contract::{AnalysisData, ValidJumpAddress};
use crate::{opcode, spec_opcode_gas, Spec, KECCAK_EMPTY};
use bytes::Bytes;
use primitive_types::H256;
use sha3::{Digest, Keccak256};
use std::sync::Arc;

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BytecodeState {
    Raw,
    Checked {
        len: usize,
    },
    Analysed {
        len: usize,
        jumptable: ValidJumpAddress,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Bytecode {
    #[cfg_attr(feature = "with-serde", serde(with = "crate::models::serde_hex_bytes"))]
    bytecode: Bytes,
    hash: H256,
    state: BytecodeState,
}

impl Default for Bytecode {
    fn default() -> Self {
        Bytecode::new()
    }
}

impl Bytecode {
    pub fn new() -> Self {
        // bytecode with one STOP opcode
        Bytecode {
            bytecode: vec![0].into(),
            hash: KECCAK_EMPTY,
            state: BytecodeState::Analysed {
                len: 0,
                jumptable: ValidJumpAddress::new(Arc::new(Vec::new()), 0),
            },
        }
    }

    pub fn new_raw(bytecode: Bytes) -> Self {
        let hash = if bytecode.is_empty() {
            KECCAK_EMPTY
        } else {
            H256::from_slice(Keccak256::digest(&bytecode).as_slice())
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
    pub unsafe fn new_raw_with_hash(bytecode: Bytes, hash: H256) -> Self {
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
    pub unsafe fn new_checked(bytecode: Bytes, len: usize, hash: Option<H256>) -> Self {
        let hash = match hash {
            None if len == 0 => KECCAK_EMPTY,
            None => H256::from_slice(Keccak256::digest(&bytecode).as_slice()),
            Some(hash) => hash,
        };
        Self {
            bytecode,
            hash,
            state: BytecodeState::Checked { len },
        }
    }

    /// Create new analysed bytecode
    ///
    /// # Safety
    /// Same as new_checked, bytecode needs to end with STOP (0x00) opcode as checked bytecode assumes
    /// that it is safe to iterate over bytecode without checking length.
    /// And that ValidJumpAddress is valid.
    pub unsafe fn new_analysed(
        bytecode: Bytes,
        len: usize,
        jumptable: ValidJumpAddress,
        hash: Option<H256>,
    ) -> Self {
        let hash = match hash {
            None if len == 0 => KECCAK_EMPTY,
            None => H256::from_slice(Keccak256::digest(&bytecode).as_slice()),
            Some(hash) => hash,
        };
        Self {
            bytecode,
            hash,
            state: BytecodeState::Analysed { len, jumptable },
        }
    }

    pub fn bytes(&self) -> &Bytes {
        &self.bytecode
    }

    pub fn hash(&self) -> H256 {
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

    pub fn to_analysed<SPEC: Spec>(self) -> Self {
        let hash = self.hash;
        let (bytecode, len) = match self.state {
            BytecodeState::Raw => {
                let len = self.bytecode.len();
                let checked = self.to_checked();
                (checked.bytecode, len)
            }
            BytecodeState::Checked { len } => (self.bytecode, len),
            _ => return self,
        };
        let jumptable = Self::analyze::<SPEC>(bytecode.as_ref());

        Self {
            bytecode,
            hash,
            state: BytecodeState::Analysed { len, jumptable },
        }
    }

    pub fn lock<SPEC: Spec>(self) -> BytecodeLocked {
        let Bytecode {
            bytecode,
            hash,
            state,
        } = self.to_analysed::<SPEC>();
        if let BytecodeState::Analysed { len, jumptable } = state {
            BytecodeLocked {
                bytecode,
                len,
                hash,
                jumptable,
            }
        } else {
            unreachable!("to_analysed transforms state to analysed");
        }
    }

    /// Analyze bytecode to get jumptable and gas blocks.
    fn analyze<SPEC: Spec>(code: &[u8]) -> ValidJumpAddress {
        let opcode_gas = spec_opcode_gas(SPEC::SPEC_ID);

        let mut analysis = ValidJumpAddress {
            first_gas_block: 0,
            analysis: Arc::new(vec![AnalysisData::none(); code.len()]),
        };
        let jumps = Arc::get_mut(&mut analysis.analysis).unwrap();

        let mut index = 0;
        let mut gas_in_block: u32 = 0;
        let mut block_start: usize = 0;

        // first gas block
        while index < code.len() {
            let opcode = *code.get(index).unwrap();
            let info = opcode_gas.get(opcode as usize).unwrap();
            analysis.first_gas_block += info.get_gas();

            index += if info.is_push() {
                ((opcode - opcode::PUSH1) + 2) as usize
            } else {
                1
            };

            if info.is_gas_block_end() {
                block_start = index - 1;
                if info.is_jump() {
                    jumps.get_mut(block_start).unwrap().set_is_jump();
                }
                break;
            }
        }

        while index < code.len() {
            let opcode = *code.get(index).unwrap();
            let info = opcode_gas.get(opcode as usize).unwrap();
            gas_in_block += info.get_gas();

            if info.is_gas_block_end() {
                if info.is_jump() {
                    jumps.get_mut(index).unwrap().set_is_jump();
                }
                jumps
                    .get_mut(block_start)
                    .unwrap()
                    .set_gas_block(gas_in_block);
                block_start = index;
                gas_in_block = 0;
                index += 1;
            } else {
                index += if info.is_push() {
                    ((opcode - opcode::PUSH1) + 2) as usize
                } else {
                    1
                };
            }
        }
        if gas_in_block != 0 {
            jumps
                .get_mut(block_start)
                .unwrap()
                .set_gas_block(gas_in_block);
        }
        analysis
    }
}

pub struct BytecodeLocked {
    bytecode: Bytes,
    len: usize,
    hash: H256,
    jumptable: ValidJumpAddress,
}

impl BytecodeLocked {
    pub fn as_ptr(&self) -> *const u8 {
        self.bytecode.as_ptr()
    }
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn unlock(self) -> Bytecode {
        Bytecode {
            bytecode: self.bytecode,
            hash: self.hash,
            state: BytecodeState::Analysed {
                len: self.len,
                jumptable: self.jumptable,
            },
        }
    }
    pub fn bytecode(&self) -> &[u8] {
        self.bytecode.as_ref()
    }

    pub fn original_bytecode_slice(&self) -> &[u8] {
        &self.bytecode.as_ref()[..self.len]
    }

    pub fn jumptable(&self) -> &ValidJumpAddress {
        &self.jumptable
    }
}
