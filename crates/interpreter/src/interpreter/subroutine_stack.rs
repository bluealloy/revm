use std::vec::Vec;

use crate::interpreter_types::SubRoutineStack;

/// Function(Sub Routine) return frame in eof
///
/// Needed information for returning from a function.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SubRoutineReturnFrame {
    /// The index of the code container that this frame is executing.
    pub idx: usize,
    /// The program counter where frame execution should continue.
    pub pc: usize,
}

impl SubRoutineReturnFrame {
    /// Return new function frame.
    pub fn new(idx: usize, pc: usize) -> Self {
        Self { idx, pc }
    }
}

/// Function Stack
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SubRoutineImpl {
    pub return_stack: Vec<SubRoutineReturnFrame>,
    pub current_code_idx: usize,
}

impl SubRoutineImpl {
    /// Returns new function stack.
    pub fn new() -> Self {
        Self {
            return_stack: Vec::new(),
            current_code_idx: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.return_stack.len()
    }

    pub fn is_empty(&self) -> bool {
        self.return_stack.is_empty()
    }

    /// Return stack length
    pub fn return_stack_len(&self) -> usize {
        self.return_stack.len()
    }

    /// Sets current_code_idx, this is needed for JUMPF opcode.
    pub fn set_current_code_idx(&mut self, idx: usize) {
        self.current_code_idx = idx;
    }
}

impl SubRoutineStack for SubRoutineImpl {
    fn len(&self) -> usize {
        self.return_stack.len()
    }

    fn routine_idx(&self) -> usize {
        self.current_code_idx
    }

    fn push(&mut self, program_counter: usize, new_idx: usize) -> bool {
        if self.return_stack.len() >= 1024 {
            return false;
        }
        self.return_stack.push(SubRoutineReturnFrame {
            idx: self.current_code_idx,
            pc: program_counter,
        });
        self.current_code_idx = new_idx;
        true
    }

    fn pop(&mut self) -> Option<usize> {
        self.return_stack.pop().map(|i| {
            self.current_code_idx = i.idx;
            i.pc
        })
    }

    fn set_routine_idx(&mut self, idx: usize) {
        self.current_code_idx = idx;
    }
}
