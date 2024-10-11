use std::vec::Vec;

/// Function return frame.
/// Needed information for returning from a function.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FunctionReturnFrame {
    /// The index of the code container that this frame is executing.
    pub idx: usize,
    /// The program counter where frame execution should continue.
    pub pc: usize,
}

impl FunctionReturnFrame {
    /// Return new function frame.
    pub fn new(idx: usize, pc: usize) -> Self {
        Self { idx, pc }
    }
}

/// Function Stack
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FunctionStack {
    pub return_stack: Vec<FunctionReturnFrame>,
    pub current_code_idx: usize,
}

impl FunctionStack {
    /// Returns new function stack.
    pub fn new() -> Self {
        Self {
            return_stack: Vec::new(),
            current_code_idx: 0,
        }
    }

    /// Pushes a new frame to the stack. and sets current_code_idx to new value.
    pub fn push(&mut self, program_counter: usize, new_idx: usize) {
        self.return_stack.push(FunctionReturnFrame {
            idx: self.current_code_idx,
            pc: program_counter,
        });
        self.current_code_idx = new_idx;
    }

    /// Return stack length
    pub fn return_stack_len(&self) -> usize {
        self.return_stack.len()
    }

    /// Pops a frame from the stack and sets current_code_idx to the popped frame's idx.
    pub fn pop(&mut self) -> Option<FunctionReturnFrame> {
        self.return_stack
            .pop()
            .inspect(|frame| self.current_code_idx = frame.idx)
    }

    /// Sets current_code_idx, this is needed for JUMPF opcode.
    pub fn set_current_code_idx(&mut self, idx: usize) {
        self.current_code_idx = idx;
    }
}
