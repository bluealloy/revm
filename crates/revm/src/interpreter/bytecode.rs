use bytes::Bytes;
use crate::{Spec, spec_opcode_gas, opcode};
use super::contract::{AnalysisData, ValidJumpAddress};


#[derive(Clone,Debug)]
pub enum BytecodeState {
    Raw,
    Checked { len: usize },
    Analysed { len: usize, jumptable: ValidJumpAddress },
}

#[derive(Clone,Debug)]
pub struct Bytecode {
    bytecode: Bytes,
    state: BytecodeState,
}

impl Bytecode {
    pub fn new_raw(bytecode: Bytes) -> Self {
        Self {
            bytecode,
            state: BytecodeState::Raw,
        }
    }

    pub fn new_checked(bytecode: Bytes, len: usize) -> Self {
        Self {
            bytecode,
            state: BytecodeState::Checked { len },
        }
    }

    pub fn new_analysed(bytecode: Bytes, len: usize, jumptable: ValidJumpAddress) -> Self {
        Self {
            bytecode,
            state: BytecodeState::Analysed { len, jumptable },
        }
    }

    pub fn len(&self) -> usize {
        match self.state {
            BytecodeState::Raw => self.bytecode.len(),
            BytecodeState::Checked { len, .. } => len,
            BytecodeState::Analysed { len, .. } => len,
        }
    }

    pub fn to_checked(mut self) -> Self {
        match self.state {
            BytecodeState::Raw => {
                let len = self.bytecode.len();
                let mut bytecode = Vec::with_capacity(len + 33);
                bytecode.extend(self.bytecode);
                bytecode.resize(len + 33, 0);

                Self {
                    bytecode: bytecode.into(),
                    state: BytecodeState::Checked { len },
                }
            }
            _ => self,
        }
    }

    pub fn to_analyzed<SPEC: Spec>(mut self) -> Self {
        let (bytecode, len) = match self.state {
            BytecodeState::Raw => {
                let len = self.bytecode.len();
                let mut bytecode = Vec::with_capacity(len + 33);
                bytecode.extend(self.bytecode);
                bytecode.resize(len + 33, 0);
                (bytecode.into(), len)
            }
            BytecodeState::Checked { len } => (self.bytecode, len),
            _ => return self,
        };

        let (jumptable,bytecode) = Self::analyze::<SPEC>(bytecode.as_ref());

        Self {
            bytecode: bytecode.into(),
            state: BytecodeState::Analysed { len, jumptable },
        }
    }

    pub fn lock<SPEC: Spec>(&self) -> BytecodeLocked {
        let locked = self.clone();
        let Bytecode{bytecode, state} = locked.to_analyzed::<SPEC>();
        if let BytecodeState::Analysed { len, jumptable } = state {
            BytecodeLocked {
                bytecode,
                len,
                jumptable,
            }
        } else {
            unreachable!("to_analyzed transforms state to analysed");
        }
    }

        /// Create a new valid mapping from given code bytes.
    /// it gives back ValidJumpAddress and size od needed paddings.
    fn analyze<SPEC: Spec>(code: &[u8]) -> (ValidJumpAddress, Vec<u8>) {
        let mut jumps: Vec<AnalysisData> = Vec::with_capacity(code.len() + 33);
        // padding of PUSH32 size plus one for stop
        jumps.resize(code.len() + 33, AnalysisData::none());
        //let opcode_gas = LONDON_OPCODES;
        let opcode_gas = spec_opcode_gas(SPEC::SPEC_ID);
        let mut index = 0;
        let mut first_gas_block: u64 = 0;
        let mut block_start: usize = 0;
        // first gas block

        while index < code.len() {
            let opcode = unsafe { *code.get_unchecked(index) };
            let info = unsafe { opcode_gas.get_unchecked(opcode as usize) };
            first_gas_block += info.gas;

            index += if info.is_push {
                ((opcode - opcode::PUSH1) + 2) as usize
            } else {
                1
            };

            if info.is_gas_block_end {
                block_start = index - 1;
                if info.is_jump {
                    unsafe {
                        jumps.get_unchecked_mut(block_start).is_jumpdest = true;
                    }
                }
                break;
            }
        }

        let mut gas_in_block: u64 = 0;
        while index < code.len() {
            let opcode = unsafe { *code.get_unchecked(index) };
            let info = unsafe { opcode_gas.get_unchecked(opcode as usize) };
            gas_in_block += info.gas;

            if info.is_gas_block_end {
                if info.is_jump {
                    unsafe {
                        jumps.get_unchecked_mut(index).is_jumpdest = true;
                    }
                }
                unsafe {
                    jumps.get_unchecked_mut(block_start).gas_block = gas_in_block;
                }
                block_start = index;
                gas_in_block = 0;
            }

            index += if info.is_push {
                ((opcode - opcode::PUSH1) + 2) as usize
            } else {
                1
            };
        }
        if gas_in_block != 0 {
            unsafe {
                jumps.get_unchecked_mut(block_start).gas_block = gas_in_block;
            }
        }
        let padding = index - code.len();
        // +1 is for forced STOP opcode at the end of contract, it is precausion
        // if there is none, and if there is STOP our additional opcode will do nothing.
        //jumps.resize(jumps.len() + padding + 1, AnalysisData::none());
        let mut code = code.to_vec();
        code.resize(code.len() + padding + 1, 0);

        (ValidJumpAddress::new(jumps, first_gas_block), code)
    }
}

pub struct BytecodeLocked {
    bytecode: Bytes,
    len: usize,
    jumptable: ValidJumpAddress,
}

impl BytecodeLocked {
    pub fn as_ptr(&self) -> *const u8 {
        self.bytecode.as_ptr()
    }
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn unlock(self) -> Bytecode {
        Bytecode {
            bytecode: self.bytecode,
            state: BytecodeState::Analysed { len: self.len, jumptable: self.jumptable }
        }
    }

    pub fn original_bytecode_slice(&self) -> &[u8] {
        &self.bytecode.as_ref()[..self.len]
    }

    pub fn jumptable(&self) -> &ValidJumpAddress {
        &self.jumptable
    }
}