//! Module that contains the verification logic for the EOF bytecode.

use crate::{
    eof::{CodeInfo, Eof, EofDecodeError},
    opcode::{self, OPCODE_INFO},
    utils::{read_i16, read_u16},
};
use primitives::{
    constants::{MAX_INITCODE_SIZE, STACK_LIMIT},
    Bytes,
};

use core::{convert::identity, mem};
use std::{borrow::Cow, fmt, vec, vec::Vec};

/// Decodes `raw` into an [`Eof`] container and validates it.
pub fn validate_raw_eof(raw: Bytes) -> Result<Eof, EofError> {
    validate_raw_eof_inner(raw, Some(CodeType::Initcode))
}

/// Decodes `raw` into an [`Eof`] container and validates it.
#[inline]
pub fn validate_raw_eof_inner(
    raw: Bytes,
    first_code_type: Option<CodeType>,
) -> Result<Eof, EofError> {
    if raw.len() > MAX_INITCODE_SIZE {
        return Err(EofError::Decode(EofDecodeError::InvalidEOFSize));
    }
    let eof = Eof::decode(raw)?;
    validate_eof_inner(&eof, first_code_type)?;
    Ok(eof)
}

/// Fully validates an [`Eof`] container.
///
/// Only place where validation happen is in Creating Transaction.
///
/// Because of that we are assuming [CodeType] is [ReturnContract][CodeType::Initcode].
///
/// Note: If needed we can make a flag that would assume [ReturnContract][CodeType::Initcode]..
pub fn validate_eof(eof: &Eof) -> Result<(), EofError> {
    validate_eof_inner(eof, Some(CodeType::Initcode))
}

/// Fully validates an [`Eof`] container. If first_code_type is None it will be auto deduced
/// in verification process.
#[inline]
pub fn validate_eof_inner(eof: &Eof, first_code_type: Option<CodeType>) -> Result<(), EofError> {
    // Data needs to be filled in the first container.
    if !eof.body.is_data_filled {
        return Err(EofError::Validation(EofValidationError::DataNotFilled));
    }
    if eof.body.container_section.is_empty() {
        validate_eof_codes(eof, first_code_type)?;
        return Ok(());
    }

    let mut stack = Vec::with_capacity(4);
    stack.push((Cow::Borrowed(eof), first_code_type));

    while let Some((eof, code_type)) = stack.pop() {
        // Validate the current container.
        let tracker_containers = validate_eof_codes(&eof, code_type)?;
        // Decode subcontainers and push them to the stack.
        for (container, code_type) in eof
            .body
            .container_section
            .iter()
            .zip(tracker_containers.into_iter())
        {
            stack.push((Cow::Owned(Eof::decode(container.clone())?), Some(code_type)));
        }
    }

    Ok(())
}

/// Validates an [`Eof`] structure, without recursing into containers.
///
/// Returns a list of all sub containers that are accessed.
#[inline]
pub fn validate_eof_codes(
    eof: &Eof,
    this_code_type: Option<CodeType>,
) -> Result<Vec<CodeType>, EofValidationError> {
    if eof.body.code_section.len() != eof.body.code_info.len() {
        return Err(EofValidationError::InvalidCodeInfo);
    }

    if eof.body.code_section.is_empty() {
        // No code sections. This should be already checked in decode.
        return Err(EofValidationError::NoCodeSections);
    }

    // The first code section must have a type signature
    // (0, 0x80, max_stack_height) (0 inputs non-returning function)
    let first_types = &eof.body.code_info[0];
    if first_types.inputs != 0 || !first_types.is_non_returning() {
        return Err(EofValidationError::InvalidCodeInfo);
    }

    // Tracking access of code and sub containers.
    let mut tracker: AccessTracker = AccessTracker::new(
        this_code_type,
        eof.body.code_section.len(),
        eof.body.container_section.len(),
    );

    while let Some(index) = tracker.processing_stack.pop() {
        // Assume `index` is correct.
        let code = eof.body.code(index).unwrap();
        validate_eof_code(
            &code,
            eof.header.data_size as usize,
            index,
            eof.body.container_section.len(),
            &eof.body.code_info,
            &mut tracker,
        )?;
    }

    // Iterate over accessed codes and check if all are accessed.
    if !tracker.codes.into_iter().all(identity) {
        return Err(EofValidationError::CodeSectionNotAccessed);
    }
    // Iterate over all accessed subcontainers and check if all are accessed.
    if !tracker.subcontainers.iter().all(|i| i.is_some()) {
        return Err(EofValidationError::SubContainerNotAccessed);
    }

    if tracker.this_container_code_type == Some(CodeType::Initcode) && !eof.body.is_data_filled {
        return Err(EofValidationError::DataNotFilled);
    }

    Ok(tracker
        .subcontainers
        .into_iter()
        .map(|i| i.unwrap())
        .collect())
}

/// EOF Error
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum EofError {
    /// Decoding error.
    Decode(EofDecodeError),
    /// Validation Error.
    Validation(EofValidationError),
}

impl From<EofDecodeError> for EofError {
    fn from(err: EofDecodeError) -> Self {
        EofError::Decode(err)
    }
}

impl From<EofValidationError> for EofError {
    fn from(err: EofValidationError) -> Self {
        EofError::Validation(err)
    }
}

impl fmt::Display for EofError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EofError::Decode(e) => write!(f, "Bytecode decode error: {}", e),
            EofError::Validation(e) => write!(f, "Bytecode validation error: {}", e),
        }
    }
}

impl core::error::Error for EofError {}

/// EOF Validation Error
#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum EofValidationError {
    /// Used in testing to indicate that the bytecode validation is different from expected.
    FalsePositive,
    /// Opcode is not known. It is not defined in the opcode table.
    UnknownOpcode,
    /// Opcode is disabled in EOF. For example JUMP, JUMPI, etc
    OpcodeDisabled,
    /// Every instruction inside bytecode should be forward accessed
    ///
    /// Forward access can be a jump or sequential opcode.
    ///
    /// In case after terminal opcode there should be a forward jump.
    InstructionNotForwardAccessed,
    /// Bytecode is too small and is missing immediate bytes for instruction
    MissingImmediateBytes,
    /// Bytecode is too small and is missing immediate bytes for instruction
    ///
    /// Similar to [`MissingImmediateBytes`][EofValidationError::MissingImmediateBytes] but for special case of RJUMPV immediate bytes.
    MissingRJUMPVImmediateBytes,
    /// Invalid jump into immediate bytes
    JumpToImmediateBytes,
    /// Invalid jump into immediate bytes
    BackwardJumpToImmediateBytes,
    /// MaxIndex in RJUMPV can't be zero. Zero max index makes it RJUMPI
    RJUMPVZeroMaxIndex,
    /// Jump with zero offset would make a jump to next opcode, it does not make sense
    JumpZeroOffset,
    /// EOFCREATE points to container out of bounds
    EOFCREATEInvalidIndex,
    /// CALLF section out of bounds
    CodeSectionOutOfBounds,
    /// CALLF to non returning function is not allowed
    CALLFNonReturningFunction,
    /// CALLF stack overflow
    StackOverflow,
    /// JUMPF needs to have enough outputs
    JUMPFEnoughOutputs,
    /// JUMPF Stack
    JUMPFStackHigherThanOutputs,
    /// DATA load out of bounds
    DataLoadOutOfBounds,
    /// RETF biggest stack num more then outputs
    RETFBiggestStackNumMoreThenOutputs,
    /// Stack requirement is more than smallest stack items
    StackUnderflow,
    /// Jump out of bounds
    JumpUnderflow,
    /// Jump to out of bounds
    JumpOverflow,
    /// Backward jump should have same smallest and biggest stack items
    BackwardJumpBiggestNumMismatch,
    /// Backward jump should have same smallest and biggest stack items
    BackwardJumpSmallestNumMismatch,
    /// Last instruction should be terminating
    LastInstructionNotTerminating,
    /// Code section not accessed
    CodeSectionNotAccessed,
    /// Types section invalid
    InvalidCodeInfo,
    /// First types section is invalid
    /// It should have inputs 0 and outputs `0x80`
    InvalidFirstCodeInfo,
    /// Max stack element mismatch
    MaxStackMismatch,
    /// No code sections present
    NoCodeSections,
    /// Sub container called in two different modes
    ///
    /// Check [`CodeType`] for more information.
    SubContainerCalledInTwoModes,
    /// Sub container not accessed
    SubContainerNotAccessed,
    /// Data size needs to be filled for [ReturnContract][CodeType::Initcode] type
    DataNotFilled,
    /// Section is marked as non-returning but has either RETF or
    /// JUMPF to returning section opcodes
    NonReturningSectionIsReturning,
}

/// Tracker status of verification of code sections and subcontainers.
/// Used in validating EOF container.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AccessTracker {
    /// This code type
    pub this_container_code_type: Option<CodeType>,
    /// Vector of accessed codes.
    pub codes: Vec<bool>,
    /// Stack of codes section that needs to be processed.
    pub processing_stack: Vec<usize>,
    /// Code accessed by subcontainer and expected subcontainer first code type.
    /// EOF code can be invoked in EOFCREATE mode or used in RETURNCONTRACT opcode.
    /// if SubContainer is called from EOFCREATE it needs to be ReturnContract type.
    /// If SubContainer is called from RETURNCONTRACT it needs to be ReturnOrStop type.
    ///
    /// None means it is not accessed.
    pub subcontainers: Vec<Option<CodeType>>,
}

impl AccessTracker {
    /// Creates a new instance with the given container type and section sizes.
    /// The first code section is marked as accessed and added to the processing stack.
    ///
    /// # Panics
    ///
    /// Panics if `codes_size` is zero.
    pub fn new(
        this_container_code_type: Option<CodeType>,
        codes_size: usize,
        subcontainers_size: usize,
    ) -> Self {
        if codes_size == 0 {
            panic!("There should be at least one code section");
        }
        let mut this = Self {
            this_container_code_type,
            codes: vec![false; codes_size],
            processing_stack: Vec::with_capacity(4),
            subcontainers: vec![None; subcontainers_size],
        };
        this.codes[0] = true;
        this.processing_stack.push(0);
        this
    }

    /// Marks a code section as accessed and adds it to the processing stack if not previously accessed.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    pub fn access_code(&mut self, index: usize) {
        let was_accessed = mem::replace(&mut self.codes[index], true);
        if !was_accessed {
            self.processing_stack.push(index);
        }
    }

    /// Sets the code type for a subcontainer. If code type is already set check if it is the same.
    /// In case of mismatch return error.
    ///
    /// # Panics
    ///
    /// Panics if the index is out of bounds.
    pub fn set_subcontainer_type(
        &mut self,
        index: usize,
        new_code_type: CodeType,
    ) -> Result<(), EofValidationError> {
        let Some(container) = self.subcontainers.get_mut(index) else {
            panic!("It should not be possible")
        };

        let Some(code_type) = container else {
            *container = Some(new_code_type);
            return Ok(());
        };

        if *code_type != new_code_type {
            return Err(EofValidationError::SubContainerCalledInTwoModes);
        }
        Ok(())
    }
}

/// Types of code sections in EOF container
///
/// Container cannot mix RETURNCONTRACT with RETURN/STOP opcodes
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CodeType {
    /// Code that initializes and returns a contract.
    Initcode,
    /// Runtime code that ends with RETURN or STOP opcodes.
    Runtime,
}

impl CodeType {
    /// Returns `true` of the code is initcode.
    pub fn is_initcode(&self) -> bool {
        matches!(self, CodeType::Initcode)
    }
}

impl fmt::Display for EofValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::FalsePositive => "False positive",
            Self::UnknownOpcode => "Opcode is not known",
            Self::OpcodeDisabled => "Opcode is disabled",
            Self::InstructionNotForwardAccessed => "Should have forward jump",
            Self::MissingImmediateBytes => "Bytecode is missing bytes",
            Self::MissingRJUMPVImmediateBytes => "Bytecode is missing bytes after RJUMPV opcode",
            Self::JumpToImmediateBytes => "Invalid jump",
            Self::BackwardJumpToImmediateBytes => "Invalid backward jump",
            Self::RJUMPVZeroMaxIndex => "Used RJUMPV with zero as MaxIndex",
            Self::JumpZeroOffset => "Used JUMP with zero as offset",
            Self::EOFCREATEInvalidIndex => "EOFCREATE points to out of bound index",
            Self::CodeSectionOutOfBounds => "CALLF index is out of bounds",
            Self::CALLFNonReturningFunction => "CALLF was used on non-returning function",
            Self::StackOverflow => "CALLF stack overflow",
            Self::JUMPFEnoughOutputs => "JUMPF needs more outputs",
            Self::JUMPFStackHigherThanOutputs => "JUMPF stack is too high for outputs",
            Self::DataLoadOutOfBounds => "DATALOAD is out of bounds",
            Self::RETFBiggestStackNumMoreThenOutputs => {
                "RETF biggest stack num is more than outputs"
            }
            Self::StackUnderflow => "Stack requirement is above smallest stack items",
            Self::JumpUnderflow => "Jump destination is too low",
            Self::JumpOverflow => "Jump destination is too high",
            Self::BackwardJumpBiggestNumMismatch => {
                "Backward jump has different biggest stack item"
            }
            Self::BackwardJumpSmallestNumMismatch => {
                "Backward jump has different smallest stack item"
            }
            Self::LastInstructionNotTerminating => {
                "Last instruction of bytecode is not terminating"
            }
            Self::CodeSectionNotAccessed => "Code section was not accessed",
            Self::InvalidCodeInfo => "Invalid types section",
            Self::InvalidFirstCodeInfo => "Invalid first types section",
            Self::MaxStackMismatch => "Max stack element mismatches",
            Self::NoCodeSections => "No code sections",
            Self::SubContainerCalledInTwoModes => "Sub container called in two modes",
            Self::SubContainerNotAccessed => "Sub container not accessed",
            Self::DataNotFilled => "Data not filled",
            Self::NonReturningSectionIsReturning => "Non returning section is returning",
        };
        f.write_str(s)
    }
}

impl core::error::Error for EofValidationError {}

/// Validates that:
/// * All instructions are valid.
/// * It ends with a terminating instruction or RJUMP.
/// * All instructions are accessed by forward jumps or .
///
/// Validate stack requirements and if all codes sections are used.
pub fn validate_eof_code(
    code: &[u8],
    data_size: usize,
    this_types_index: usize,
    num_of_containers: usize,
    types: &[CodeInfo],
    tracker: &mut AccessTracker,
) -> Result<(), EofValidationError> {
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

    impl InstructionInfo {
        #[inline]
        fn mark_as_immediate(&mut self) -> Result<(), EofValidationError> {
            if self.is_jumpdest {
                // Jump to immediate bytes.
                return Err(EofValidationError::JumpToImmediateBytes);
            }
            self.is_immediate = true;
            Ok(())
        }
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

    // All bytes that are intermediate.
    let mut jumps = vec![InstructionInfo::default(); code.len()];
    let mut is_after_termination = false;

    let mut next_smallest = this_types.inputs as i32;
    let mut next_biggest = this_types.inputs as i32;

    let mut is_returning = false;

    let mut i = 0;
    // We can check validity and jump destinations in one pass.
    while i < code.len() {
        let op = code[i];
        let opcode = &OPCODE_INFO[op as usize];

        let Some(opcode) = opcode else {
            // Err unknown opcode.
            return Err(EofValidationError::UnknownOpcode);
        };

        if opcode.is_disabled_in_eof() {
            // Opcode is disabled in EOF
            return Err(EofValidationError::OpcodeDisabled);
        }

        let this_instruction = &mut jumps[i];

        // Update biggest/smallest values for next instruction only if it is not after termination.
        if !is_after_termination {
            this_instruction.smallest = core::cmp::min(this_instruction.smallest, next_smallest);
            this_instruction.biggest = core::cmp::max(this_instruction.biggest, next_biggest);
        }

        let this_instruction = *this_instruction;

        // Opcodes after termination should be accessed by forward jumps.
        if is_after_termination && !this_instruction.is_jumpdest {
            // Opcode after termination was not accessed.
            return Err(EofValidationError::InstructionNotForwardAccessed);
        }
        is_after_termination = opcode.is_terminating();

        // Mark immediate as non-jumpable. RJUMPV is special case covered later.
        if opcode.immediate_size() != 0 {
            // Check if the opcode immediate are within the bounds of the code
            if i + opcode.immediate_size() as usize >= code.len() {
                // Malfunctional code
                return Err(EofValidationError::MissingImmediateBytes);
            }

            // Mark immediate bytes as non-jumpable.
            for imm in 1..opcode.immediate_size() as usize + 1 {
                // SAFETY: Immediate size is checked above.
                jumps[i + imm].mark_as_immediate()?;
            }
        }
        // IO diff used to generate next instruction smallest/biggest value.
        let mut stack_io_diff = opcode.io_diff() as i32;
        // How many stack items are required for this opcode.
        let mut stack_requirement = opcode.inputs() as i32;
        // Additional immediate bytes for RJUMPV, it has dynamic vtable.
        let mut rjumpv_additional_immediates = 0;
        // If opcodes is RJUMP, RJUMPI or RJUMPV then this will have absolute jumpdest.
        let mut absolute_jumpdest = vec![];
        match op {
            opcode::RJUMP | opcode::RJUMPI => {
                let offset = unsafe { read_i16(code.as_ptr().add(i + 1)) } as isize;
                absolute_jumpdest = vec![offset + 3 + i as isize];
                // RJUMP is considered a terminating opcode.
            }
            opcode::RJUMPV => {
                // Code length for RJUMPV is checked with immediate size.
                let max_index = code[i + 1] as usize;
                let len = max_index + 1;
                // And max_index+1 is to get size of vtable as index starts from 0.
                rjumpv_additional_immediates = len * 2;

                // +1 is for max_index byte
                if i + 1 + rjumpv_additional_immediates >= code.len() {
                    // Malfunctional code RJUMPV vtable is not complete
                    return Err(EofValidationError::MissingRJUMPVImmediateBytes);
                }

                // Mark vtable as immediate, max_index was already marked.
                for imm in 0..rjumpv_additional_immediates {
                    // SAFETY: Immediate size is checked above.
                    jumps[i + 2 + imm].mark_as_immediate()?;
                }

                let mut jumps = Vec::with_capacity(len);
                for vtablei in 0..len {
                    let offset =
                        unsafe { read_i16(code.as_ptr().add(i + 2 + 2 * vtablei)) } as isize;
                    jumps.push(offset + i as isize + 2 + rjumpv_additional_immediates as isize);
                }
                absolute_jumpdest = jumps
            }
            opcode::CALLF => {
                let section_i: usize = unsafe { read_u16(code.as_ptr().add(i + 1)) } as usize;
                let Some(target_types) = types.get(section_i) else {
                    // Code section out of bounds.
                    return Err(EofValidationError::CodeSectionOutOfBounds);
                };

                // CALLF operand must not point to a section with 0x80 as outputs (non-returning)
                if target_types.is_non_returning() {
                    return Err(EofValidationError::CALLFNonReturningFunction);
                }
                // Stack input for this opcode is the input of the called code.
                stack_requirement = target_types.inputs as i32;
                // Stack diff depends on input/output of the called code.
                stack_io_diff = target_types.io_diff();
                // Mark called code as accessed.
                tracker.access_code(section_i);

                if this_instruction.biggest + target_types.max_stack_increase as i32
                    > STACK_LIMIT as i32
                {
                    // If stack max items + called code max stack size
                    return Err(EofValidationError::StackOverflow);
                }
            }
            opcode::JUMPF => {
                let target_index = unsafe { read_u16(code.as_ptr().add(i + 1)) } as usize;
                // Targeted code needs to have zero outputs (be non returning).
                let Some(target_types) = types.get(target_index) else {
                    // Code section out of bounds.
                    return Err(EofValidationError::CodeSectionOutOfBounds);
                };

                if this_instruction.biggest + target_types.max_stack_increase as i32
                    > STACK_LIMIT as i32
                {
                    // stack overflow
                    return Err(EofValidationError::StackOverflow);
                }
                tracker.access_code(target_index);

                if target_types.is_non_returning() {
                    // If it is not returning
                    stack_requirement = target_types.inputs as i32;
                } else {
                    is_returning = true;
                    // Check if target code produces enough outputs.
                    if this_types.outputs < target_types.outputs {
                        return Err(EofValidationError::JUMPFEnoughOutputs);
                    }

                    stack_requirement = this_types.outputs as i32 + target_types.inputs as i32
                        - target_types.outputs as i32;

                    // Stack requirement needs to more than this instruction biggest stack number.
                    if this_instruction.biggest > stack_requirement {
                        return Err(EofValidationError::JUMPFStackHigherThanOutputs);
                    }

                    // If this instruction max + target_types max is more then stack limit.
                    if this_instruction.biggest + stack_requirement > STACK_LIMIT as i32 {
                        return Err(EofValidationError::StackOverflow);
                    }
                }
            }
            opcode::EOFCREATE => {
                let index = code[i + 1] as usize;
                if index >= num_of_containers {
                    // Code section out of bounds.
                    return Err(EofValidationError::EOFCREATEInvalidIndex);
                }
                tracker.set_subcontainer_type(index, CodeType::Initcode)?;
            }
            opcode::RETURNCONTRACT => {
                let index = code[i + 1] as usize;
                if index >= num_of_containers {
                    // Code section out of bounds.
                    // TODO : Custom error
                    return Err(EofValidationError::EOFCREATEInvalidIndex);
                }
                if *tracker
                    .this_container_code_type
                    .get_or_insert(CodeType::Initcode)
                    != CodeType::Initcode
                {
                    // TODO : Make custom error
                    return Err(EofValidationError::SubContainerCalledInTwoModes);
                }
                tracker.set_subcontainer_type(index, CodeType::Runtime)?;
            }
            opcode::RETURN | opcode::STOP => {
                if *tracker
                    .this_container_code_type
                    .get_or_insert(CodeType::Runtime)
                    != CodeType::Runtime
                {
                    return Err(EofValidationError::SubContainerCalledInTwoModes);
                }
            }
            opcode::DATALOADN => {
                let index = unsafe { read_u16(code.as_ptr().add(i + 1)) } as isize;
                if data_size < 32 || index > data_size as isize - 32 {
                    // Data load out of bounds.
                    return Err(EofValidationError::DataLoadOutOfBounds);
                }
            }
            opcode::RETF => {
                stack_requirement = this_types.outputs as i32;
                // Mark section as returning.
                is_returning = true;

                if this_instruction.biggest > stack_requirement {
                    return Err(EofValidationError::RETFBiggestStackNumMoreThenOutputs);
                }
            }
            opcode::DUPN => {
                stack_requirement = code[i + 1] as i32 + 1;
            }
            opcode::SWAPN => {
                stack_requirement = code[i + 1] as i32 + 2;
            }
            opcode::EXCHANGE => {
                let imm = code[i + 1];
                let n = (imm >> 4) + 1;
                let m = (imm & 0x0F) + 1;
                stack_requirement = n as i32 + m as i32 + 1;
            }
            _ => {}
        }
        // Check if stack requirement is more than smallest stack items.
        if stack_requirement > this_instruction.smallest {
            // Opcode requirement is more than smallest stack items.
            return Err(EofValidationError::StackUnderflow);
        }

        next_smallest = this_instruction.smallest + stack_io_diff;
        next_biggest = this_instruction.biggest + stack_io_diff;

        // Check if jumpdest are correct and mark forward jumps.
        for absolute_jump in absolute_jumpdest {
            if absolute_jump < 0 {
                // Jump out of bounds.
                return Err(EofValidationError::JumpUnderflow);
            }
            if absolute_jump >= code.len() as isize {
                // Jump to out of bounds
                return Err(EofValidationError::JumpOverflow);
            }
            // Fine to cast as bounds are checked.
            let absolute_jump = absolute_jump as usize;

            let target_jump = &mut jumps[absolute_jump];
            if target_jump.is_immediate {
                // Jump target is immediate byte.
                return Err(EofValidationError::BackwardJumpToImmediateBytes);
            }

            // Needed to mark forward jumps. It does not do anything for backward jumps.
            target_jump.is_jumpdest = true;

            if absolute_jump <= i {
                // Backward jumps should have same smallest and biggest stack items.
                if target_jump.biggest != next_biggest {
                    // Wrong jumpdest.
                    return Err(EofValidationError::BackwardJumpBiggestNumMismatch);
                }
                if target_jump.smallest != next_smallest {
                    // Wrong jumpdest.
                    return Err(EofValidationError::BackwardJumpSmallestNumMismatch);
                }
            } else {
                // Forward jumps can make min even smallest size
                // While biggest num is needed to check stack overflow
                target_jump.smallest = core::cmp::min(target_jump.smallest, next_smallest);
                target_jump.biggest = core::cmp::max(target_jump.biggest, next_biggest);
            }
        }

        // Additional immediate are from RJUMPV vtable.
        i += 1 + opcode.immediate_size() as usize + rjumpv_additional_immediates;
    }

    // Error if section is returning but marked as non-returning.
    if is_returning == this_types.is_non_returning() {
        // Wrong termination.
        return Err(EofValidationError::NonReturningSectionIsReturning);
    }

    // Last opcode should be terminating
    if !is_after_termination {
        // Wrong termination.
        return Err(EofValidationError::LastInstructionNotTerminating);
    }
    // TODO : Integrate max so we dont need to iterate again
    let this_code_info = &types[this_types_index];
    let mut max_stack_requirement = 0;
    for opcode in jumps {
        max_stack_requirement = core::cmp::max(
            opcode.biggest.saturating_sub(this_code_info.inputs as i32),
            max_stack_requirement,
        );
    }

    if max_stack_requirement != this_code_info.max_stack_increase as i32 {
        // Stack overflow
        return Err(EofValidationError::MaxStackMismatch);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use primitives::hex;

    #[test]
    fn test1() {
        // result:Result { result: false, exception: Some("EOF_ConflictingStackHeight") }
        let err =
            validate_raw_eof(hex!("ef00010100040200010007ff000000008000016000e200fffc00").into());
        assert!(err.is_err(), "{err:#?}");
    }

    #[test]
    fn test2() {
        // result:Result { result: false, exception: Some("EOF_InvalidNumberOfOutputs") }
        let err =
            validate_raw_eof_inner(hex!("ef000101000c020003000400040002ff000000008000020002000100010001e30001005fe500025fe4").into(),None);
        assert!(err.is_ok(), "{err:#?}");
    }

    #[test]
    fn test3() {
        // result:Result { result: false, exception: Some("EOF_InvalidNumberOfOutputs") }
        let err =
            validate_raw_eof_inner(hex!("ef000101000c020003000400080003ff000000008000020002000503010003e30001005f5f5f5f5fe500025050e4").into(),None);
        assert_eq!(
            err,
            Err(EofError::Validation(
                EofValidationError::JUMPFStackHigherThanOutputs
            ))
        );
    }

    #[test]
    fn test4() {
        // result:Result { result: false, exception: Some("EOF_InvalidNumberOfOutputs") }
        let err = validate_raw_eof(
            hex!("ef0001010004020001000eff000000008000045f6000e100025f5f6000e1fffd00").into(),
        );
        assert_eq!(
            err,
            Err(EofError::Validation(
                EofValidationError::BackwardJumpBiggestNumMismatch
            ))
        );
    }

    #[test]
    fn test5() {
        let err = validate_raw_eof(hex!("ef00010100040200010003ff00000000800000e5ffff").into());
        assert_eq!(
            err,
            Err(EofError::Validation(
                EofValidationError::CodeSectionOutOfBounds
            ))
        );
    }

    #[test]
    fn size_limit() {
        let eof = validate_raw_eof_inner(
            hex!("ef00010100040200010003ff0001000080000130500000").into(),
            Some(CodeType::Runtime),
        );
        assert!(eof.is_ok());
    }

    #[test]
    fn test() {
        let eof = validate_raw_eof_inner(
            hex!("ef00010100040200010005ffff0300008000023a60cbee1800").into(),
            None,
        );
        assert_eq!(
            eof,
            Err(EofError::Validation(EofValidationError::DataNotFilled))
        );
    }

    #[test]
    fn unreachable_code_section() {
        let eof = validate_raw_eof_inner(
            hex!("ef000101000c020003000300010003ff000000008000000080000000800000e50001fee50002")
                .into(),
            None,
        );
        assert_eq!(
            eof,
            Err(EofError::Validation(
                EofValidationError::CodeSectionNotAccessed
            ))
        );
    }

    #[test]
    fn non_returning_sections() {
        let eof = validate_raw_eof_inner(
            hex!("ef000101000c020003000400010003ff000000008000000080000000000000e300020000e50001")
                .into(),
            Some(CodeType::Runtime),
        );
        assert_eq!(
            eof,
            Err(EofError::Validation(
                EofValidationError::NonReturningSectionIsReturning
            ))
        );
    }

    #[test]
    fn incompatible_container_kind() {
        let eof = validate_raw_eof_inner(
            hex!("ef0001010004020001000603000100000014ff0000000080000260006000ee00ef00010100040200010001040000000080000000")
                .into(),
            Some(CodeType::Runtime),
        );
        assert_eq!(
            eof,
            Err(EofError::Validation(
                EofValidationError::SubContainerCalledInTwoModes
            ))
        );
    }
}
