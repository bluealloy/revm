use revm_primitives::eof::TypesSection;
use revm_primitives::{Bytes, Eof, LegacyAnalyzedBytecode};

use crate::primitives::{
    bitvec::prelude::{bitvec, BitVec, Lsb0},
    legacy::JumpTable,
    Bytecode,
};
use crate::{opcode, OPCODE_INFO_JUMPTABLE};
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
            let mut padded_bytecode = Vec::with_capacity(len + 33);
            padded_bytecode.extend_from_slice(&bytecode);
            padded_bytecode.resize(len + 33, 0);
            (Bytes::from(padded_bytecode), len)
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

/// Validate Eof structures.
///
/// do perf test on:
/// max eof containers
/// max depth of containers.
/// bytecode iteration.
pub fn validate_eof(eof: &Eof) -> Result<(), ()> {
    // clone is cheat as it is Bytes and a header.
    let mut analyze_eof = vec![eof.clone()];

    while let Some(eof) = analyze_eof.pop() {
        // iterate over types and code
        for (types, bytes) in eof
            .body
            .types_section
            .iter()
            .zip(eof.body.code_section.iter())
        {
            types.validate()?;
        }

        // iterate over containers, convert them to Eof and add to analyze_eof
        for container in eof.body.container_section {
            let container_eof = Eof::decode(container)?;
            analyze_eof.push(container_eof);
        }
    }

    // Eof is valid
    Ok(())
}

pub fn validate_eof_bytecode(code: &[u8], types: &TypesSection) -> Result<(), ()> {
    let max_stack_size = types.inputs as u16;
    let stack_size = types.inputs as u16;

    let mut iter = code.as_ptr();
    let end = code.as_ptr().wrapping_add(code.len());

    let mut eof_table = [0; 256];

    while iter < end {
        let opcode = unsafe { *iter };
        let opcode_info = &OPCODE_INFO_JUMPTABLE[opcode as usize];

        // if opcode::JUMPDEST == opcode {
        //     // SAFETY: jumps are max length of the code
        //     unsafe { jumps.set_unchecked(iterator.offset_from(start) as usize, true) }
        //     iterator = unsafe { iterator.offset(1) };
        // } else {
        //     let push_offset = opcode.wrapping_sub(opcode::PUSH1);
        //     if push_offset < 32 {
        //         // SAFETY: iterator access range is checked in the while loop
        //         iterator = unsafe { iterator.offset((push_offset + 2) as isize) };
        //     } else {
        //         // SAFETY: iterator access range is checked in the while loop
        //         iterator = unsafe { iterator.offset(1) };
        //     }
        // }
    }

    // iterate over opcodes

    if max_stack_size != types.max_stack_size {
        return Err(());
    }

    if stack_size != types.outputs as u16 {
        return Err(());
    }

    Ok(())
}
