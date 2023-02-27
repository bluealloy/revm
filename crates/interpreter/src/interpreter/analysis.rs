use crate::opcode;
use crate::primitives::{Bytecode, BytecodeState, Bytes, B256};
use revm_primitives::JumpMap;
use bitvec::prelude::{bitvec};
use alloc::sync::Arc;

/// Perform bytecode analysis.
///
/// The analysis finds and caches valid jump destinations for later execution as an optimization step.
///
/// If the bytecode is already analyzed, it is returned as-is.
pub fn to_analysed(bytecode: Bytecode) -> Bytecode {
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
    let jump_map = analyze(bytecode.as_ref());

    Bytecode {
        bytecode,
        hash,
        state: BytecodeState::Analysed { len, jump_map },
    }
}

/// Analyzs bytecode to build a jump map.
fn analyze(code: &[u8]) -> JumpMap {
    let mut jumps = bitvec![0; code.len()];

    let mut index = 0;
    while index < code.len() {
        let opcode = *code.get(index).unwrap();

        index += match opcode {
            opcode::PUSH1..=opcode::PUSH32 => ((opcode - opcode::PUSH1) + 2) as usize,
            opcode::JUMPDEST => {
                jumps.set(index, true);
                1
            }
            _ => 1,
        };
    }

    JumpMap(Arc::new(jumps))
}

#[derive(Clone)]
pub struct BytecodeLocked {
    bytecode: Bytes,
    len: usize,
    hash: B256,
    jump_map: JumpMap,
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
        if let BytecodeState::Analysed { len, jump_map } = bytecode.state {
            Ok(BytecodeLocked {
                bytecode: bytecode.bytecode,
                len,
                hash: bytecode.hash,
                jump_map,
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
                jump_map: self.jump_map,
            },
        }
    }
    pub fn bytecode(&self) -> &[u8] {
        self.bytecode.as_ref()
    }

    pub fn original_bytecode_slice(&self) -> &[u8] {
        &self.bytecode.as_ref()[..self.len]
    }

    pub fn jump_map(&self) -> &JumpMap {
        &self.jump_map
    }
}
