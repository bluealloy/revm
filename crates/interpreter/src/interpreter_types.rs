use crate::{CallInput, Gas, InstructionResult, InterpreterAction};
use bytecode::eof::CodeInfo;
use core::cell::Ref;
use core::ops::{Deref, Range};
use primitives::{hardfork::SpecId, Address, Bytes, B256, U256};

/// Helper function to read immediates data from the bytecode
pub trait Immediates {
    #[inline]
    fn read_i16(&self) -> i16 {
        self.read_u16() as i16
    }
    fn read_u16(&self) -> u16;

    #[inline]
    fn read_i8(&self) -> i8 {
        self.read_u8() as i8
    }
    fn read_u8(&self) -> u8;

    #[inline]
    fn read_offset_i16(&self, offset: isize) -> i16 {
        self.read_offset_u16(offset) as i16
    }
    fn read_offset_u16(&self, offset: isize) -> u16;

    fn read_slice(&self, len: usize) -> &[u8];
}

/// Trait for fetching inputs of the call.
pub trait InputsTr {
    /// Returns target address of the call.
    fn target_address(&self) -> Address;
    /// Returns bytecode address of the call. For DELEGATECALL this address will be different from target address.
    /// And if initcode is called this address will be [`None`].
    fn bytecode_address(&self) -> Option<&Address>;
    /// Returns caller address of the call.
    fn caller_address(&self) -> Address;
    /// Returns input of the call.
    fn input(&self) -> &CallInput;
    /// Returns call value of the call.
    fn call_value(&self) -> U256;
}

/// Trait needed for legacy bytecode.
///
/// Used in [`bytecode::opcode::CODECOPY`] and [`bytecode::opcode::CODESIZE`] opcodes.
pub trait LegacyBytecode {
    /// Returns current bytecode original length. Used in [`bytecode::opcode::CODESIZE`] opcode.
    fn bytecode_len(&self) -> usize;
    /// Returns current bytecode original slice. Used in [`bytecode::opcode::CODECOPY`] opcode.
    fn bytecode_slice(&self) -> &[u8];
}

/// Trait for Interpreter to be able to jump
pub trait Jumps {
    /// Relative jumps does not require checking for overflow.
    fn relative_jump(&mut self, offset: isize);
    /// Absolute jumps require checking for overflow and if target is a jump destination
    /// from jump table.
    fn absolute_jump(&mut self, offset: usize);
    /// Check legacy jump destination from jump table.
    fn is_valid_legacy_jump(&mut self, offset: usize) -> bool;
    /// Returns current program counter.
    fn pc(&self) -> usize;
    /// Returns instruction opcode.
    fn opcode(&self) -> u8;
}

/// Trait for Interpreter memory operations.
pub trait MemoryTr {
    /// Sets memory data at given offset from data with a given data_offset and len.
    ///
    /// # Panics
    ///
    /// Panics if range is out of scope of allocated memory.
    fn set_data(&mut self, memory_offset: usize, data_offset: usize, len: usize, data: &[u8]);

    /// Inner clone part of memory from global context to local context.
    /// This is used to clone calldata to memory.
    ///
    /// # Panics
    ///
    /// Panics if range is out of scope of allocated memory.
    fn set_data_from_global(
        &mut self,
        memory_offset: usize,
        data_offset: usize,
        len: usize,
        data_range: Range<usize>,
    );

    /// Memory slice with global range. This range
    ///
    /// # Panics
    ///
    /// Panics if range is out of scope of allocated memory.
    fn global_slice(&self, range: Range<usize>) -> Ref<'_, [u8]>;

    /// Offset of local context of memory.
    fn local_memory_offset(&self) -> usize;

    /// Sets memory data at given offset.
    ///
    /// # Panics
    ///
    /// Panics if range is out of scope of allocated memory.
    fn set(&mut self, memory_offset: usize, data: &[u8]);

    /// Returns memory size.
    fn size(&self) -> usize;

    /// Copies memory data from source to destination.
    ///
    /// # Panics
    /// Panics if range is out of scope of allocated memory.
    fn copy(&mut self, destination: usize, source: usize, len: usize);

    /// Memory slice with range
    ///
    /// # Panics
    ///
    /// Panics if range is out of scope of allocated memory.
    fn slice(&self, range: Range<usize>) -> Ref<'_, [u8]>;

    /// Memory slice len
    ///
    /// Uses [`slice`][MemoryTr::slice] internally.
    fn slice_len(&self, offset: usize, len: usize) -> impl Deref<Target = [u8]> + '_ {
        self.slice(offset..offset + len)
    }

    /// Resizes memory to new size
    ///
    /// # Note
    ///
    /// It checks memory limits.
    fn resize(&mut self, new_size: usize) -> bool;
}

/// Returns EOF containers. Used by [`bytecode::opcode::RETURNCONTRACT`] and [`bytecode::opcode::EOFCREATE`] opcodes.
pub trait EofContainer {
    /// Returns EOF container at given index.
    fn eof_container(&self, index: usize) -> Option<&Bytes>;
}

/// Handles EOF introduced sub routine calls.
pub trait SubRoutineStack {
    /// Returns sub routine stack length.
    fn len(&self) -> usize;

    /// Returns `true` if sub routine stack is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns current sub routine index.
    fn routine_idx(&self) -> usize;

    /// Sets new code section without touching subroutine stack.
    ///
    /// This is used for [`bytecode::opcode::JUMPF`] opcode. Where
    /// tail call is performed.
    fn set_routine_idx(&mut self, idx: usize);

    /// Pushes a new frame to the stack and new code index.
    fn push(&mut self, old_program_counter: usize, new_idx: usize) -> bool;

    /// Pops previous subroutine, sets previous code index and returns program counter.
    fn pop(&mut self) -> Option<usize>;
}

/// Functions needed for Interpreter Stack operations.
pub trait StackTr {
    /// Returns stack length.
    fn len(&self) -> usize;

    /// Returns `true` if stack is empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Pushes values to the stack.
    ///
    /// Returns `true` if push was successful, `false` if stack overflow.
    ///
    /// # Note
    /// Error is internally set in interpreter.
    #[must_use]
    fn push(&mut self, value: U256) -> bool;

    /// Pushes B256 value to the stack.
    ///
    /// Internally converts B256 to U256 and then calls [`StackTr::push`].
    #[must_use]
    fn push_b256(&mut self, value: B256) -> bool {
        self.push(value.into())
    }

    /// Pops value from the stack.
    #[must_use]
    fn popn<const N: usize>(&mut self) -> Option<[U256; N]>;

    /// Pop N values from the stack and return top value.
    #[must_use]
    fn popn_top<const POPN: usize>(&mut self) -> Option<([U256; POPN], &mut U256)>;

    /// Returns top value from the stack.
    #[must_use]
    fn top(&mut self) -> Option<&mut U256> {
        self.popn_top::<0>().map(|(_, top)| top)
    }

    /// Pops one value from the stack.
    #[must_use]
    fn pop(&mut self) -> Option<U256> {
        self.popn::<1>().map(|[value]| value)
    }

    /// Pops address from the stack.
    ///
    /// Internally call [`StackTr::pop`] and converts [`U256`] into [`Address`].
    #[must_use]
    fn pop_address(&mut self) -> Option<Address> {
        self.pop().map(|value| Address::from(value.to_be_bytes()))
    }

    /// Exchanges two values on the stack.
    ///
    /// Indexes are based from the top of the stack.
    ///
    /// Returns `true` if swap was successful, `false` if stack underflow.
    #[must_use]
    fn exchange(&mut self, n: usize, m: usize) -> bool;

    /// Duplicates the `N`th value from the top of the stack.
    ///
    /// Index is based from the top of the stack.
    ///
    /// Returns `true` if duplicate was successful, `false` if stack underflow.
    #[must_use]
    fn dup(&mut self, n: usize) -> bool;
}

/// EOF data fetching.
pub trait EofData {
    /// Returns EOF data.
    fn data(&self) -> &[u8];
    /// Returns EOF data slice.
    fn data_slice(&self, offset: usize, len: usize) -> &[u8];
    /// Returns EOF data size.
    fn data_size(&self) -> usize;
}

/// EOF code info.
pub trait EofCodeInfo {
    /// Returns code information containing stack information.
    fn code_info(&self, idx: usize) -> Option<&CodeInfo>;

    /// Returns program counter at the start of code section.
    fn code_section_pc(&self, idx: usize) -> Option<usize>;
}

/// Returns return data.
pub trait ReturnData {
    /// Returns return data.
    fn buffer(&self) -> &Bytes;

    /// Sets return buffer.
    fn set_buffer(&mut self, bytes: Bytes);

    /// Clears return buffer.
    fn clear(&mut self) {
        self.set_buffer(Bytes::new());
    }
}

pub trait LoopControl {
    fn set_instruction_result(&mut self, result: InstructionResult);
    fn set_next_action(&mut self, action: InterpreterAction, result: InstructionResult);
    fn gas(&self) -> &Gas;
    fn gas_mut(&mut self) -> &mut Gas;
    fn instruction_result(&self) -> InstructionResult;
    fn take_next_action(&mut self) -> InterpreterAction;
}

pub trait RuntimeFlag {
    fn is_static(&self) -> bool;
    fn is_eof(&self) -> bool;
    fn is_eof_init(&self) -> bool;
    fn spec_id(&self) -> SpecId;
}

pub trait Interp {
    type Instruction;
    type Action;

    fn run(&mut self, instructions: &[Self::Instruction; 256]) -> Self::Action;
}

pub trait InterpreterTypes {
    type Stack: StackTr;
    type Memory: MemoryTr;
    type Bytecode: Jumps + Immediates + LegacyBytecode + EofData + EofContainer + EofCodeInfo;
    type ReturnData: ReturnData;
    type Input: InputsTr;
    type SubRoutineStack: SubRoutineStack;
    type Control: LoopControl;
    type RuntimeFlag: RuntimeFlag;
    type Extend;
    type Output;
}
