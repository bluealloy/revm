use crate::{
    interpreter::Interpreter,
    primitives::{Address, Output},
    JournalCheckpoint,
};
use core::ops::Range;
use revm_interpreter::{CallOutcome, CreateOutcome, Gas, InstructionResult, InterpreterResult};
use std::boxed::Box;

/// Call CallStackFrame.
#[derive(Debug)]
pub struct CallFrame {
    /// Call frame has return memory range where output will be stored.
    pub return_memory_range: Range<usize>,
    /// Frame data
    pub frame_data: FrameData,
}

#[derive(Debug)]
pub struct CreateFrame {
    /// Create frame has a created address.
    pub created_address: Address,
    /// Frame data
    pub frame_data: FrameData,
}

#[derive(Debug)]
pub struct FrameData {
    /// Journal checkpoint
    pub checkpoint: JournalCheckpoint,
    /// Interpreter
    pub interpreter: Interpreter,
}

/// Call stack frame.
#[derive(Debug)]
pub enum Frame {
    Call(Box<CallFrame>),
    Create(Box<CreateFrame>),
}

pub enum FrameResult {
    Call(CallOutcome),
    Create(CreateOutcome),
}

impl FrameResult {
    /// Casts frame result to interpreter result.
    #[inline]
    pub fn into_interpreter_result(self) -> InterpreterResult {
        match self {
            FrameResult::Call(outcome) => outcome.result,
            FrameResult::Create(outcome) => outcome.result,
        }
    }

    /// Returns execution output.
    #[inline]
    pub fn output(&self) -> Output {
        match self {
            FrameResult::Call(outcome) => Output::Call(outcome.result.output.clone()),
            FrameResult::Create(outcome) => {
                Output::Create(outcome.result.output.clone(), outcome.address)
            }
        }
    }

    /// Returns reference to gas.
    #[inline]
    pub fn gas(&self) -> &Gas {
        match self {
            FrameResult::Call(outcome) => &outcome.result.gas,
            FrameResult::Create(outcome) => &outcome.result.gas,
        }
    }

    /// Returns mutable reference to interpreter result.
    #[inline]
    pub fn gas_mut(&mut self) -> &mut Gas {
        match self {
            FrameResult::Call(outcome) => &mut outcome.result.gas,
            FrameResult::Create(outcome) => &mut outcome.result.gas,
        }
    }

    /// Returns reference to interpreter result.
    #[inline]
    pub fn interpreter_result(&self) -> &InterpreterResult {
        match self {
            FrameResult::Call(outcome) => &outcome.result,
            FrameResult::Create(outcome) => &outcome.result,
        }
    }

    /// Returns mutable reference to interpreter result.
    #[inline]
    pub fn interpreter_result_mut(&mut self) -> &InterpreterResult {
        match self {
            FrameResult::Call(outcome) => &mut outcome.result,
            FrameResult::Create(outcome) => &mut outcome.result,
        }
    }

    /// Return Instruction result.
    #[inline]
    pub fn instruction_result(&self) -> InstructionResult {
        self.interpreter_result().result
    }
}

/// Contains either a frame or a result.
pub enum FrameOrResult {
    /// Boxed call or create frame.
    Frame(Frame),
    /// Call or create result.
    Result(FrameResult),
}

impl Frame {
    pub fn new_create(
        created_address: Address,
        checkpoint: JournalCheckpoint,
        interpreter: Interpreter,
    ) -> Self {
        Frame::Create(Box::new(CreateFrame {
            created_address,
            frame_data: FrameData {
                checkpoint,
                interpreter,
            },
        }))
    }

    pub fn new_call(
        return_memory_range: Range<usize>,
        checkpoint: JournalCheckpoint,
        interpreter: Interpreter,
    ) -> Self {
        Frame::Call(Box::new(CallFrame {
            return_memory_range,
            frame_data: FrameData {
                checkpoint,
                interpreter,
            },
        }))
    }

    /// Returns true if frame is call frame.
    pub fn is_call(&self) -> bool {
        matches!(self, Frame::Call { .. })
    }

    /// Returns true if frame is create frame.
    pub fn is_create(&self) -> bool {
        matches!(self, Frame::Create { .. })
    }

    /// Returns created address if frame is create otherwise returns None.
    pub fn created_address(&self) -> Option<Address> {
        match self {
            Frame::Create(create_frame) => Some(create_frame.created_address),
            _ => None,
        }
    }

    /// Takes frame and returns frame data.
    pub fn into_frame_data(self) -> FrameData {
        match self {
            Frame::Call(call_frame) => call_frame.frame_data,
            Frame::Create(create_frame) => create_frame.frame_data,
        }
    }

    /// Returns reference to frame data.
    pub fn frame_data(&self) -> &FrameData {
        match self {
            Self::Call(call_frame) => &call_frame.frame_data,
            Self::Create(create_frame) => &create_frame.frame_data,
        }
    }

    /// Returns mutable reference to frame data.
    pub fn frame_data_mut(&mut self) -> &mut FrameData {
        match self {
            Self::Call(call_frame) => &mut call_frame.frame_data,
            Self::Create(create_frame) => &mut create_frame.frame_data,
        }
    }

    /// Returns a reference to the interpreter.
    pub fn interpreter(&self) -> &Interpreter {
        &self.frame_data().interpreter
    }

    /// Returns a mutable reference to the interpreter.
    pub fn interpreter_mut(&mut self) -> &mut Interpreter {
        &mut self.frame_data_mut().interpreter
    }
}

impl FrameOrResult {
    /// Creates new create frame.
    pub fn new_create_frame(
        created_address: Address,
        checkpoint: JournalCheckpoint,
        interpreter: Interpreter,
    ) -> Self {
        Self::Frame(Frame::new_create(created_address, checkpoint, interpreter))
    }

    /// Creates new call frame.
    pub fn new_call_frame(
        return_memory_range: Range<usize>,
        checkpoint: JournalCheckpoint,
        interpreter: Interpreter,
    ) -> Self {
        Self::Frame(Frame::new_call(
            return_memory_range,
            checkpoint,
            interpreter,
        ))
    }

    /// Creates new create result.
    pub fn new_create_result(
        interpreter_result: InterpreterResult,
        address: Option<Address>,
    ) -> Self {
        FrameOrResult::Result(FrameResult::Create(CreateOutcome {
            result: interpreter_result,
            address,
        }))
    }

    pub fn new_call_result(
        interpreter_result: InterpreterResult,
        memory_offset: Range<usize>,
    ) -> Self {
        FrameOrResult::Result(FrameResult::Call(CallOutcome {
            result: interpreter_result,
            memory_offset,
        }))
    }
}
