use crate::{
    instructions::utility::{read_i16, read_u16},
    opcode,
    primitives::{
        bitvec::prelude::{bitvec, BitVec, Lsb0},
        eof::TypesSection,
        legacy::JumpTable,
        Bytecode, Bytes, Eof, LegacyAnalyzedBytecode,
    },
    OpCode, OPCODE_INFO_JUMPTABLE, STACK_LIMIT,
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

/// Validates that:
/// * All instructions are valid.
/// * It ends with a terminating instruction or RJUMP.
///
pub fn validate_eof_bytecode(
    code: &[u8],
    accessed_codes: &mut [bool],
    types: &[TypesSection],
) -> Result<(), ()> {
    #[derive(Copy, Default, Clone)]
    pub struct BytecodeMark {
        /// Is immediate byte, jumps can't happen on this part of code.
        is_immediate: bool,
        /// Have forward jump to this opcode. Used to check if opcode
        /// after termination is accessed.
        has_forward_jump: bool,
    }

    // all bytes that are intermediate.
    let mut jumps = vec![BytecodeMark::default(); code.len()];

    let mut is_after_termination = false;

    let mut i = 0;
    // We can check validity and jump destinations in one pass.
    while i < code.len() {
        let op = code[i];
        let opcode_info = &OPCODE_INFO_JUMPTABLE[op as usize];
        let this_jump = jumps[i];

        // Unknown opcode
        let Some(opcode) = opcode_info else {
            // err unknown opcode.
            return Err(());
        };

        if !opcode.is_eof {
            // Opcode is disabled in EOF
            return Err(());
        }

        // Opcodes after termination should be accessed by forward jumps.
        if is_after_termination && this_jump.has_forward_jump {
            // opcode after termination was not accessed.
            return Err(());
        }
        is_after_termination = opcode.is_terminating_opcode;

        // mark immediates as non-jumpable. RJUMPV is special case covered later.
        if opcode.immediate_size != 0 {
            // check if the opcode immediates are within the bounds of the code
            if i + opcode.immediate_size as usize > code.len() {
                // Malfunctional code
                return Err(());
            }

            // mark immediate bytes as non-jumpable.
            for imm in 1..opcode.immediate_size as usize + 1 {
                // SAFETY: immediate size is checked above.
                let jumptable = &mut jumps[imm];
                if jumptable.has_forward_jump {
                    // There is a jump to the immediate bytes.
                    return Err(());
                }
                jumptable.is_immediate = true;
            }
        }
        let mut additional_immediates = 0;
        // get absolute jumpdest from RJUMP, RJUMPI and RJUMPV
        let absolute_jumpdest = match op {
            opcode::RJUMP | opcode::RJUMPI => {
                let offset = read_i16(unsafe { code.as_ptr().add(i + 1) }) as isize;
                if offset == 0 {
                    // jump immediate instruction is not allowed.
                    return Err(());
                }
                vec![offset + 3 + i as isize]
            }
            opcode::RJUMPV => {
                let max_index = code[i + 1] as usize;
                additional_immediates = (1 + max_index) * 2;

                // Max index can't be zero as it becomes RJUMPI.
                if max_index == 0 {
                    return Err(());
                }

                // +1 is for max_index byte, and max_index+1 is to get size of vtable.
                if i + 1 + additional_immediates >= code.len() {
                    // Malfunctional code RJUMPV vtable is not complete
                    return Err(());
                }

                let mut jumps = Vec::with_capacity(max_index);
                for vtablei in 0..max_index {
                    let offset =
                        read_i16(unsafe { code.as_ptr().add(i + 2 + 2 * vtablei) }) as isize;
                    if offset == 0 {
                        // jump immediate instruction is not allowed.
                        return Err(());
                    }
                    jumps[vtablei] = offset + i as isize + 2 + additional_immediates as isize;
                }
                jumps
            }
            _ => vec![],
        };

        // check if jumpdest are correct.
        for absolute_jump in absolute_jumpdest {
            if absolute_jump < 0 {
                // jump out of bounds.
                return Err(());
            }
            if absolute_jump > code.len() as isize {
                // jump to out of bounds
                return Err(());
            }
            // fine to cast as bound are checked.
            let absolute_jump = absolute_jump as usize;

            let target_jump = &mut jumps[absolute_jump];
            if target_jump.is_immediate {
                // Jump target is immediate byte.
                return Err(());
            }
            // for previous jumps we already marked them in is immediate.
            target_jump.has_forward_jump = true;
        }

        match op {
            opcode::CALLF => {
                let section_i = read_u16(unsafe { code.as_ptr().add(i + 1) }) as usize;
                // targeted code needs to have zero outputs (be non returning).
                let Some(next_section) = types.get(section_i) else {
                    // code section out of bounds.
                    return Err(());
                };

                if next_section.outputs == EOF_NON_RETURNING_FUNCTION {
                    // callf to non returning function is not allowed
                    return Err(());
                }
                accessed_codes[section_i] = true;
            }
            opcode::RETF => {
                // check if it is returning. TODO here
            }
            _ => {}
        }

        // if let Some(jump) = opcode_info.jump {
        //     eof_table[jump as usize] += 1;
        // }

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

    // last opcode should be terminating
    if !is_after_termination {
        // wrong termination.
        return Err(());
    }

    Ok(())
}

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
pub fn validate_eof_stack_requirement(
    codes: &[u8],
    this_types_index: usize,
    types: &[TypesSection],
) -> Result<(), ()> {
    #[derive(Copy, Clone)]
    struct StackIO {
        pub min: u16,
        pub max: u16,
    }

    impl StackIO {
        pub fn next(&self, diff: i16) -> Self {
            Self {
                min: self.min + diff as u16,
                max: self.max + diff as u16,
            }
        }
    }

    impl Default for StackIO {
        fn default() -> Self {
            Self {
                min: u16::MAX,
                max: 0,
            }
        }
    }

    // Stack access information for each instruction section.
    let mut code_stack_access: Vec<StackIO> = vec![Default::default(); codes.len()];

    // Set first instruction min and max stack requirement as a this code section input.
    let this_types = &types[this_types_index];
    code_stack_access[0] = StackIO {
        min: this_types.inputs as u16,
        max: this_types.inputs as u16,
    };

    let max_stack_height = 0;
    let mut i = 0;
    while i < codes.len() {
        let opcode = codes[i];

        let Some(info) = OpCode::new(opcode).map(|i| i.info()) else {
            panic!("Opcode validity is checked.")
        };

        let mut stack_i = info.inputs as u16;
        let mut stack_io_diff = info.io_diff() as i16;

        let stack_io = code_stack_access[i];

        // Jump over intermediate data and set min/max to zero.
        match opcode {
            opcode::CALLF => {
                let code_id = read_u16(unsafe { codes.as_ptr().add(i + 1) }) as usize;
                let types = &types[code_id];
                // stack input for this opcode is the input of the called code.
                stack_i = types.inputs as u16;

                // we decrement types.inputs as they are considered send to the called code.
                // and included in types.max_stack_size.
                if stack_io.max - stack_i + types.max_stack_size > STACK_LIMIT as u16 {
                    // if stack max items + called code max stack size
                    return Err(());
                }
                stack_io_diff = types.io_diff() as i16;
            }
            opcode::JUMPF => {
                let code_id = read_u16(unsafe { codes.as_ptr().add(i + 1) }) as usize;

                let target_types = &types[code_id];

                // we decrement types.inputs as they are considered send to the called code.
                // and included in types.max_stack_size.
                if stack_io.max - target_types.inputs as u16 + target_types.max_stack_size
                    > STACK_LIMIT as u16
                {
                    // stack overflow
                    return Err(());
                }

                stack_io_diff = 0;
                if target_types.outputs == 0 {
                    // if it is not returning
                    stack_i = target_types.inputs as u16;
                } else {
                    // check if target code produces enough outputs.
                    if target_types.outputs < this_types.outputs {
                        return Err(());
                    }

                    // TOOD(EOF) check overflows.
                    stack_i = (target_types.outputs as i16 + this_types.io_diff()) as u16;

                    // if this instruction max + target_types max is more then stack limit.
                    if stack_io.max + stack_i > STACK_LIMIT as u16 {
                        return Err(());
                    }
                }
            }
            opcode::RETF => {
                stack_i = this_types.outputs as u16;
                if stack_io.max > stack_i {
                    // stack_higher_than_outputs_required
                    return Err(());
                }
            }
            opcode::DUPN => {
                stack_i = codes[i + 1] as u16 + 1;
                stack_io_diff = 1;
            }
            opcode::SWAPN => {
                stack_i = codes[i + 1] as u16 + 2;
            }
            opcode::EXCHANGE => {
                let imm = codes[i + 1];
                let n = (imm >> 4) + 1;
                let m = (imm & 0x0F) + 1;
                stack_i = n as u16 + m as u16;
            }
            _ => {}
        }

        if stack_io.min < stack_i {
            // should have at least min items for stack input
            return Err(());
        }

        // next item stack io;
        let mut next_stack_io = stack_io.next(stack_io_diff);

        let mut imm_size = info.immediate_size as usize;
        if opcode == opcode::RJUMPV {
            // code validation is already done and we can access codes[workitem + 1] safely.
            imm_size += codes[i + 1] as usize * 2;
        }

        // Nulify max stack if it is a terminating opcode.
        if info.is_terminating_opcode {
            next_stack_io.max = 0;
            next_stack_io.min = 0;
        }

        // next instruction index.
        i += imm_size + 1;

        // check if opcode is terminating or it is RJUMP (to previous dest).
        if info.is_terminating_opcode || opcode == opcode::RJUMP {
            // if it is not a jump instruction, we set next stack io.
            code_stack_access[i + 1] = next_stack_io;
        }

        // if next instruction is out of bounds, break. Terminal instructions are already handled.
        if i >= codes.len() {
            break;
        }

        //match opcode {}

        // check next instruction, break if it is out of bounds.
    }

    // Iterate over accessed code, error on not accessed opcode and return max stack requirement.
    let mut max_stack_requirement = 0;
    for opcode in code_stack_access {
        if opcode.min == u16::MAX {
            // opcode not accessed.
            return Err(());
        }
        max_stack_requirement = core::cmp::max(opcode.max, max_stack_requirement);
    }
    Ok(())
}
