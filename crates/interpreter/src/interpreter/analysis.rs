use revm_primitives::LegacyAnalyzedBytecode;

use crate::opcode;
use crate::primitives::{
    bitvec::prelude::{bitvec, BitVec, Lsb0},
    legacy::JumpTable,
    Bytecode,
};
use std::sync::Arc;

/// Perform bytecode analysis.
///
/// The analysis finds and caches valid jump destinations for later execution as an optimization step.
///
/// If the bytecode is already analyzed, it is returned as-is.
#[inline]
pub fn to_analysed(bytecode: Bytecode) -> Bytecode {
    let (bytes, len) = match bytecode {
        Bytecode::LegacyRaw(bytecode) => {
            let len = bytecode.len();
            (bytecode, len)
        }
        n => return n,
    };
    let jump_table = analyze(bytes.as_ref());

    Bytecode::LegacyAnalyzed(LegacyAnalyzedBytecode::new(bytes, len, jump_table))
}

/// Analyze bytecode to build a jump map.
fn analyze(code: &[u8]) -> JumpTable {
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

    JumpTable(Arc::new(jumps))
}
