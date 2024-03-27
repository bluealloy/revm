use revm_primitives::HashSet;

use crate::{
    instructions::utility::{read_i16, read_u16},
    opcode,
    primitives::{
        bitvec::prelude::{bitvec, BitVec, Lsb0},
        eof::TypesSection,
        legacy::JumpTable,
        Bytecode, Bytes, Eof, LegacyAnalyzedBytecode,
    },
    OPCODE_INFO_JUMPTABLE, STACK_LIMIT,
};
use std::{sync::Arc, vec, vec::Vec};

const EOF_NON_RETURNING_FUNCTION: u8 = 0x80;

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

pub fn validate_raw_eof(bytecode: Bytes) -> Result<Eof, EofError> {
    let eof = Eof::decode(bytecode).map_err(|_| EofError::EofDecode)?;
    validate_eof(&eof)?;
    Ok(eof)
}

/// Validate Eof structures.
///
/// do perf test on:
/// max eof containers
/// max depth of containers.
/// bytecode iteration.
pub fn validate_eof(eof: &Eof) -> Result<(), EofError> {
    // clone is cheat as it is Bytes and a header.
    let mut queue = vec![eof.clone()];

    while let Some(eof) = queue.pop() {
        // iterate over types
        for types in &eof.body.types_section {
            types
                .validate()
                .map_err(|_| EofError::InvalidTypesSection)?;
        }
        validate_eof_codes(&eof)?;
        // iterate over containers, convert them to Eof and add to analyze_eof
        for container in eof.body.container_section {
            queue.push(Eof::decode(container).map_err(|_| EofError::EofDecode)?);
        }
    }

    // Eof is valid
    Ok(())
}

/// Validate EOF
pub fn validate_eof_codes(eof: &Eof) -> Result<(), EofError> {
    let mut queued_codes = vec![false; eof.body.code_section.len()];
    // first section is default one.
    queued_codes[0] = true;
    // start validation from code section 0.
    let mut queue = vec![0];
    while let Some(index) = queue.pop() {
        let code = &eof.body.code_section[index];
        let accessed_codes = validate_eof_code(
            &code,
            eof.header.data_size as usize,
            index,
            &eof.body.types_section,
        )?;

        // queue accessed codes.
        accessed_codes.into_iter().for_each(|i| {
            if !queued_codes[i] {
                queued_codes[i] = true;
                queue.push(i);
            }
        });
    }
    // iterate over accessed codes and check if all are accessed.
    if queued_codes.into_iter().find(|&x| x == false).is_some() {
        return Err(EofError::CodeSectionNotAccessed);
    }

    Ok(())
}

/*

//0x6001800100
0x6001 PUSH1
80 DUP1
01 ADD
00 STOP

// 0xef0001010004020001000504000000008000026001800100
0xef00 magic
01 version
01 kind
0004 04 size
02 kind
0001 num of codes
0005 size of code
04 kind data
0000 size of data
00 terminator
00 inputs
80 non returning fn
0002 max stack elements
6001800100

0x6001 PUSH0
80
80
80
80
80
80
f1
00

*/

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum EofError {
    TEST,
    /// Opcode is not known. It is not defined in the opcode table.
    UnknownOpcode,
    /// Opcode is disabled in EOF. For example JUMP, JUMPI, etc.
    OpcodeDisabled,
    /// Every instruction inside bytecode should be forward accessed.
    /// Forward access can be a jump or sequential opcode.
    /// In case after terminal opcode there should be a forward jump.
    InstructionNotForwardAccessed,
    /// Bytecode is too small and is missing immediate bytes for instruction.
    MissingImmediateBytes,
    /// Similar to [`MissingImmediateBytes`] but for special case of RJUMPV immediate bytes.
    MissingRJUMPVImmediateBytes,
    /// Invalid jump into immediate bytes.
    JumpToImmediateBytes,
    /// Invalid jump into immediate bytes.
    BackwardJumpToImmediateBytes,
    /// MaxIndex in RJUMPV can't be zero. Zero max index makes it RJUMPI.
    RJUMPVZeroMaxIndex,
    /// Jump with zero offset would make a jump to next opcode, it does not make sense.
    JumpZeroOffset,
    /// CALLF section out of bounds.
    CodeSectionOutOfBounds,
    /// CALLF to non returning function is not allowed.
    CALLFNonReturningFunction,
    /// CALLF stack overflow.
    StackOverflow,
    /// JUMPF needs to have enough outputs.
    JUMPFEnoughOutputs,
    /// DATA load out of bounds.
    DataLoadOutOfBounds,
    /// TODO(EOF) check this error.
    RETFBiggestStackNumMoreThenOutputs,
    /// Stack requirement is more than smallest stack items.
    StackUnderflow,
    /// Jump out of bounds.
    JumpUnderflow,
    /// Jump to out of bounds.
    JumpOverflow,
    /// Backward jump should have same smallest and biggest stack items.
    BackwardJumpBiggestNumMismatch,
    /// Backward jump should have same smallest and biggest stack items.
    BackwardJumpSmallestNumMismatch,
    /// Last instruction should be terminating.
    LastInstructionNotTerminating,
    /// Code section not accessed.
    CodeSectionNotAccessed,
    /// Types section invalid
    InvalidTypesSection,
    /// EofDecode error,
    EofDecode,
    /// Max stack element mismatch.
    MaxStackMismatch,
}

/*
0x6000 PUSH1
e200
00035b5b00600160015500


 */

/// Validates that:
/// * All instructions are valid.
/// * It ends with a terminating instruction or RJUMP.
/// * All instructions are accessed by forward jumps or .
///
/// Validate stack requirements and if all codes sections are used.
///
/// TODO mark accessed Types/Codes
///
/// Preconditions:
/// * Jump destinations are valid.
/// * All instructions are valid and well formed.
/// * All instruction is accessed by forward jumps.
/// * Bytecode is valid and ends with terminating instruction.
///
/// Preconditions are checked in `validate_eof_bytecode`.
pub fn validate_eof_code(
    code: &[u8],
    data_size: usize,
    this_types_index: usize,
    types: &[TypesSection],
) -> Result<HashSet<usize>, EofError> {
    let mut accessed_codes = HashSet::<usize>::new();
    let this_types = &types[this_types_index];

    #[derive(Debug, Copy, Clone)]
    struct InstructionInfo {
        /// Is immediate byte, jumps can't happen on this part of code.
        is_immediate: bool,
        /// Have forward jump to this opcode. Used to check if opcode
        /// after termination is accessed.
        is_jumpdest: bool,
        /// Smallest number of stack items accessed by jumps or sequential opcodes.
        smallest: i32,
        /// Biggest number of stack items accessed by jumps or sequential opcodes.
        biggest: i32,
    }

    impl Default for InstructionInfo {
        fn default() -> Self {
            Self {
                is_immediate: false,
                is_jumpdest: false,
                smallest: i32::MAX,
                biggest: i32::MIN,
            }
        }
    }

    // all bytes that are intermediate.
    let mut jumps = vec![InstructionInfo::default(); code.len()];
    let mut is_after_termination = false;

    let mut next_smallest = this_types.inputs as i32;
    let mut next_biggest = this_types.inputs as i32;

    let mut i = 0;
    // We can check validity and jump destinations in one pass.
    while i < code.len() {
        let op = code[i];
        let opcode = &OPCODE_INFO_JUMPTABLE[op as usize];
        let this_instruction = &mut jumps[i];
        this_instruction.smallest = core::cmp::min(this_instruction.smallest, next_smallest);
        this_instruction.biggest = core::cmp::max(this_instruction.biggest, next_biggest);

        let this_instruction = *this_instruction;

        let Some(opcode) = opcode else {
            // err unknown opcode.
            return Err(EofError::UnknownOpcode);
        };

        if !opcode.is_eof {
            // Opcode is disabled in EOF
            return Err(EofError::OpcodeDisabled);
        }

        // Opcodes after termination should be accessed by forward jumps.
        if is_after_termination && this_instruction.is_jumpdest {
            // opcode after termination was not accessed.
            return Err(EofError::InstructionNotForwardAccessed);
        }
        is_after_termination = opcode.is_terminating_opcode;

        // mark immediate as non-jumpable. RJUMPV is special case covered later.
        if opcode.immediate_size != 0 {
            // check if the opcode immediate are within the bounds of the code
            if i + opcode.immediate_size as usize >= code.len() {
                // Malfunctional code
                return Err(EofError::MissingImmediateBytes);
            }

            // mark immediate bytes as non-jumpable.
            for imm in 1..opcode.immediate_size as usize + 1 {
                // SAFETY: immediate size is checked above.
                let jumptable = &mut jumps[i + imm];
                if jumptable.is_jumpdest {
                    // There is a jump to the immediate bytes.
                    return Err(EofError::JumpToImmediateBytes);
                }
                jumptable.is_immediate = true;
            }
        }
        // IO diff used to generate next instruction smallest/biggest value.
        let mut stack_io_diff = opcode.io_diff() as i32;
        // how many stack items are required for this opcode.
        let mut stack_requirement = opcode.inputs as i32;
        // additional immediate bytes for RJUMPV, it has dynamic vtable.
        let mut rjumpv_additional_immediates = 0;
        // If opcodes is RJUMP, RJUMPI or RJUMPV then this will have absolute jumpdest.
        let mut absolute_jumpdest = vec![];
        match op {
            opcode::RJUMP | opcode::RJUMPI => {
                let offset = unsafe { read_i16(code.as_ptr().add(i + 1)) } as isize;
                if offset == 0 {
                    // jump immediate instruction is not allowed.
                    return Err(EofError::JumpToImmediateBytes);
                }
                absolute_jumpdest = vec![offset + 3 + i as isize]
            }
            opcode::RJUMPV => {
                // code length for RJUMPV is checked with immediate size.
                let max_index = code[i + 1] as usize;
                // and max_index+1 is to get size of vtable as index starts from 0.
                rjumpv_additional_immediates = (1 + max_index) * 2;

                // Max index can't be zero as it becomes RJUMPI.
                if max_index == 0 {
                    return Err(EofError::RJUMPVZeroMaxIndex);
                }

                // +1 is for max_index byte
                if i + 1 + rjumpv_additional_immediates >= code.len() {
                    // Malfunctional code RJUMPV vtable is not complete
                    return Err(EofError::MissingRJUMPVImmediateBytes);
                }

                let mut jumps = Vec::with_capacity(max_index);
                for vtablei in 0..max_index {
                    let offset =
                        unsafe { read_i16(code.as_ptr().add(i + 2 + 2 * vtablei)) } as isize;
                    if offset == 0 {
                        // jump immediate instruction is not allowed.
                        return Err(EofError::JumpZeroOffset);
                    }
                    jumps.push(offset + i as isize + 2 + rjumpv_additional_immediates as isize);
                }
                absolute_jumpdest = jumps
            }
            opcode::CALLF => {
                let section_i = unsafe { read_u16(code.as_ptr().add(i + 1)) } as usize;
                let Some(target_types) = types.get(section_i) else {
                    // code section out of bounds.
                    return Err(EofError::CodeSectionOutOfBounds);
                };

                if target_types.outputs == EOF_NON_RETURNING_FUNCTION {
                    // callf to non returning function is not allowed
                    return Err(EofError::CALLFNonReturningFunction);
                }
                // stack input for this opcode is the input of the called code.
                stack_requirement = target_types.inputs as i32;
                // stack diff depends on input/output of the called code.
                stack_io_diff = target_types.io_diff() as i32;
                // mark called code as accessed.
                accessed_codes.insert(section_i);

                // we decrement by `types.inputs` as they are considered as send
                // to the called code and included in types.max_stack_size.
                if this_instruction.biggest - stack_requirement + target_types.max_stack_size as i32
                    > STACK_LIMIT as i32
                {
                    // if stack max items + called code max stack size
                    return Err(EofError::StackOverflow);
                }
            }
            opcode::JUMPF => {
                let target_index = unsafe { read_u16(code.as_ptr().add(i + 1)) } as usize;
                // targeted code needs to have zero outputs (be non returning).
                let Some(target_types) = types.get(target_index) else {
                    // code section out of bounds.
                    return Err(EofError::CodeSectionOutOfBounds);
                };

                // we decrement types.inputs as they are considered send to the called code.
                // and included in types.max_stack_size.
                if this_instruction.biggest - target_types.inputs as i32
                    + target_types.max_stack_size as i32
                    > STACK_LIMIT as i32
                {
                    // stack overflow
                    return Err(EofError::StackOverflow);
                }

                accessed_codes.insert(target_index);

                if target_types.outputs == EOF_NON_RETURNING_FUNCTION {
                    // if it is not returning
                    stack_requirement = target_types.inputs as i32;
                    // if it is not returning JUMPF becomes terminating opcode.
                    is_after_termination = true;
                } else {
                    // check if target code produces enough outputs.
                    if target_types.outputs < this_types.outputs {
                        return Err(EofError::JUMPFEnoughOutputs);
                    }

                    // TODO(EOF) stack requirements for this opcode.
                    stack_requirement = (target_types.outputs as i32 + this_types.io_diff()) as i32;

                    // if this instruction max + target_types max is more then stack limit.
                    if this_instruction.biggest + stack_requirement > STACK_LIMIT as i32 {
                        return Err(EofError::StackOverflow);
                    }
                }
            }
            opcode::DATALOADN => {
                let index = unsafe { read_u16(code.as_ptr().add(i + 1)) } as isize;
                if data_size < 32 || index > data_size as isize - 32 {
                    // data load out of bounds.
                    return Err(EofError::DataLoadOutOfBounds);
                }
            }
            opcode::RETF => {
                stack_requirement = this_types.outputs as i32;
                if this_instruction.biggest > stack_requirement {
                    // stack_higher_than_outputs_required
                    // TODO(EOF) Why is this here. Why are we erroring if biggest number
                    // is more than outputs?
                    return Err(EofError::RETFBiggestStackNumMoreThenOutputs);
                }
            }
            opcode::DUPN => {
                stack_requirement = code[i + 1] as i32 + 1;
                stack_io_diff = 1;
            }
            opcode::SWAPN => {
                stack_requirement = code[i + 1] as i32 + 2;
            }
            opcode::EXCHANGE => {
                let imm = code[i + 1];
                let n = (imm >> 4) + 1;
                let m = (imm & 0x0F) + 1;
                stack_requirement = n as i32 + m as i32;
            }
            _ => {}
        }
        // check if stack requirement is more than smallest stack items.
        if stack_requirement > this_instruction.smallest {
            // opcode requirement is more than smallest stack items.
            return Err(EofError::StackUnderflow);
        }

        // check if jumpdest are correct and mark forward jumps.
        for absolute_jump in absolute_jumpdest {
            if absolute_jump < 0 {
                // jump out of bounds.
                return Err(EofError::JumpUnderflow);
            }
            if absolute_jump >= code.len() as isize {
                // jump to out of bounds
                return Err(EofError::JumpOverflow);
            }
            // fine to cast as bounds are checked.
            let absolute_jump = absolute_jump as usize;

            let target_jump = &mut jumps[absolute_jump];
            if target_jump.is_immediate {
                // Jump target is immediate byte.
                return Err(EofError::BackwardJumpToImmediateBytes);
            }

            // needed to mark forward jumps. It does not do anything for backward jumps.
            target_jump.is_jumpdest = true;

            if absolute_jump < i {
                // backward jumps should have same smallest and biggest stack items.
                if this_instruction.biggest != target_jump.biggest {
                    // wrong jumpdest.
                    return Err(EofError::BackwardJumpBiggestNumMismatch);
                }
                if this_instruction.smallest != target_jump.smallest {
                    // wrong jumpdest.
                    return Err(EofError::BackwardJumpSmallestNumMismatch);
                }
            } else {
                // forward jumps can make min even smallest size
                // while biggest num is needed to check stack overflow
                target_jump.smallest =
                    core::cmp::min(target_jump.smallest, this_instruction.smallest);
                target_jump.biggest = core::cmp::max(target_jump.biggest, this_instruction.biggest);
            }
        }
        //println!("stack_io_diff: {}", stack_io_diff);
        next_smallest = this_instruction.smallest + stack_io_diff;
        next_biggest = this_instruction.smallest + stack_io_diff;
        // println!(
        //     "next_smallest: {} next_biggest: {}",
        //     next_smallest, next_biggest
        // );
        // additional immediate are from RJUMPV vtable.
        i += 1 + opcode.immediate_size as usize + rjumpv_additional_immediates;
    }

    // last opcode should be terminating
    if !is_after_termination {
        // wrong termination.
        return Err(EofError::LastInstructionNotTerminating);
    }

    // TODO integrate max so we dont need to iterate again
    let mut max_stack_requirement = 0;
    for opcode in jumps {
        max_stack_requirement = core::cmp::max(opcode.biggest, max_stack_requirement);
    }

    if max_stack_requirement != types[this_types_index].max_stack_size as i32 {
        // stack overflow
        return Err(EofError::MaxStackMismatch);
    }

    Ok(accessed_codes)
}
