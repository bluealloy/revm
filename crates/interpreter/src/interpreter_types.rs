use bytecode::eof::TypesSection;
use specification::hardfork::SpecId;

use crate::{Gas, InstructionResult, InterpreterAction};
use core::ops::{Deref, Range};
use primitives::{Address, Bytes, B256, U256};

/// Helper function to read immediates data from the bytecode
pub trait Immediates {
    fn read_i16(&self) -> i16;
    fn read_u16(&self) -> u16;

    fn read_i8(&self) -> i8;
    fn read_u8(&self) -> u8;

    fn read_offset_i16(&self, offset: isize) -> i16;
    fn read_offset_u16(&self, offset: isize) -> u16;

    fn read_slice(&self, len: usize) -> &[u8];
}

pub trait InputsT {
    fn target_address(&self) -> Address;
    fn caller_address(&self) -> Address;
    fn input(&self) -> &[u8];
    fn call_value(&self) -> U256;
}

pub trait LegacyBytecode {
    fn bytecode_len(&self) -> usize;
    fn bytecode_slice(&self) -> &[u8];
}

/// Trait for interpreter to be able to jump
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

pub trait MemoryTr {
    fn set_data(&mut self, memory_offset: usize, data_offset: usize, len: usize, data: &[u8]);
    fn set(&mut self, memory_offset: usize, data: &[u8]);

    fn size(&self) -> usize;
    fn copy(&mut self, destination: usize, source: usize, len: usize);

    /// Memory slice with range
    ///
    /// # Panics
    /// Panics if range is out of scope of allocated memory.
    fn slice(&self, range: Range<usize>) -> impl Deref<Target = [u8]> + '_;

    /// Memory slice len
    ///
    /// Uses [`slice`][MemoryTr::slice] internally.
    fn slice_len(&self, offset: usize, len: usize) -> impl Deref<Target = [u8]> + '_ {
        self.slice(offset..offset + len)
    }

    /// Resizes memory to new size
    ///
    /// # Note
    /// It checks memory limits.
    fn resize(&mut self, new_size: usize) -> bool;
}

pub trait EofContainer {
    fn eof_container(&self, index: usize) -> Option<&Bytes>;
}

pub trait SubRoutineStack {
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn routine_idx(&self) -> usize;

    /// Sets new code section without touching subroutine stack.
    fn set_routine_idx(&mut self, idx: usize);

    /// Pushes a new frame to the stack and new code index.
    fn push(&mut self, old_program_counter: usize, new_idx: usize) -> bool;

    /// Pops previous subroutine, sets previous code index and returns program counter.
    fn pop(&mut self) -> Option<usize>;

    // /// Returns code info from EOF body.
    // fn eof_code_info(&self, idx: usize) -> Option<&TypesSection>;
}

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

pub trait EofData {
    fn data(&self) -> &[u8];
    fn data_slice(&self, offset: usize, len: usize) -> &[u8];
    fn data_size(&self) -> usize;
}

pub trait EofCodeInfo {
    /// Returns code information containing stack information.
    fn code_section_info(&self, idx: usize) -> Option<&TypesSection>;

    /// Returns program counter at the start of code section.
    fn code_section_pc(&self, idx: usize) -> Option<usize>;
}

pub trait ReturnData {
    fn buffer(&self) -> &[u8];
    fn buffer_mut(&mut self) -> &mut Bytes;
}

pub trait LoopControl {
    fn set_instruction_result(&mut self, result: InstructionResult);
    fn set_next_action(&mut self, action: InterpreterAction, result: InstructionResult);
    fn gas(&mut self) -> &mut Gas;
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
    type Input: InputsT;
    type SubRoutineStack: SubRoutineStack;
    type Control: LoopControl;
    type RuntimeFlag: RuntimeFlag;
    type Extend;
}
