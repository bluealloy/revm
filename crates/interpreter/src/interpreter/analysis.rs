use crate::primitives::{AnalysisData, Bytecode, BytecodeState, Spec, ValidJumpAddress, B256};
use crate::{opcode, spec_opcode_gas};
use bytes::Bytes;
use std::sync::Arc;

pub fn to_analysed<SPEC: Spec>(bytecode: Bytecode) -> Bytecode {
    let hash = bytecode.hash;
    let (bytecode, len) = match bytecode.state {
        BytecodeState::Raw => {
            let len = bytecode.bytecode.len();
            let checked = bytecode.to_checked();
            (checked.bytecode, len)
        }
        BytecodeState::Checked { len } => (bytecode.bytecode, len),
        _ => return bytecode,
    };
    let jumptable = analyze::<SPEC>(bytecode.as_ref());

    Bytecode {
        bytecode,
        hash,
        state: BytecodeState::Analysed { len, jumptable },
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

#[derive(Clone)]
pub struct BytecodeLocked {
    bytecode: Bytes,
    len: usize,
    hash: B256,
    jumptable: ValidJumpAddress,
}

impl Default for BytecodeLocked {
    fn default() -> Self {
        Bytecode::default()
            .try_into()
            .expect("Bytecode default is analysed code")
    }
}

impl TryFrom<Bytecode> for BytecodeLocked {
    type Error = ();

    fn try_from(bytecode: Bytecode) -> Result<Self, Self::Error> {
        if let BytecodeState::Analysed { len, jumptable } = bytecode.state {
            Ok(BytecodeLocked {
                bytecode: bytecode.bytecode,
                len,
                hash: bytecode.hash,
                jumptable,
            })
        } else {
            Err(())
        }
    }
}

impl BytecodeLocked {
    pub fn as_ptr(&self) -> *const u8 {
        self.bytecode.as_ptr()
    }
    pub fn len(&self) -> usize {
        self.len
    }

    pub fn hash(&self) -> B256 {
        self.hash
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
