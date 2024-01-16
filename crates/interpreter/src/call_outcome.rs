use crate::{Gas, InstructionResult, InterpreterResult};
use core::ops::Range;
use revm_primitives::Bytes;

pub struct CallOutcome {
    pub interpreter_result: InterpreterResult,
    pub memory_return_offset: Range<usize>,
}

impl CallOutcome {
    pub fn new(interpreter_result: InterpreterResult, memory_return_offset: Range<usize>) -> Self {
        Self {
            interpreter_result,
            memory_return_offset,
        }
    }
    pub fn instruction_result(&self) -> &InstructionResult {
        &self.interpreter_result.result
    }
    pub fn gas(&self) -> Gas {
        self.interpreter_result.gas
    }
    pub fn output(&self) -> &Bytes {
        &self.interpreter_result.output
    }
    pub fn memory_offset_start(&self) -> usize {
        self.memory_return_offset.start
    }
    pub fn memory_length(&self) -> usize {
        self.memory_return_offset.len()
    }
}
