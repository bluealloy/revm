use crate::{
    interpreter::{Interpreter, InterpreterResult},
    primitives::Address,
    JournalCheckpoint,
};
use alloc::boxed::Box;
use core::ops::Range;

/// Call CallStackFrame.
#[derive(Debug)]
pub struct CallStackFrame {
    /// Journal checkpoint
    pub checkpoint: JournalCheckpoint,
    /// Interpreter
    pub interpreter: Interpreter,
    /// Frame data
    pub frame_data: FrameData,
}

/// Specific data for call or create frame.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FrameData {
    /// Call frame has return memory range where output will be stored.
    Call { return_memory_range: Range<usize> },
    /// Create frame has a created address.
    Create { created_address: Address },
}

/// Contains either a frame or a result.
pub enum FrameOrResult {
    /// Boxed stack frame
    Frame(Box<CallStackFrame>),
    /// Interpreter result
    Result(InterpreterResult),
}

impl CallStackFrame {
    /// Returns true if frame is call frame.
    pub fn is_call(&self) -> bool {
        matches!(self.frame_data, FrameData::Call { .. })
    }

    /// Returns true if frame is create frame.
    pub fn is_create(&self) -> bool {
        matches!(self.frame_data, FrameData::Create { .. })
    }

    /// Returns created address if frame is create otherwise returns None.
    pub fn created_address(&self) -> Option<Address> {
        match self.frame_data {
            FrameData::Create { created_address } => Some(created_address),
            _ => None,
        }
    }
}

impl FrameOrResult {
    /// Returns new frame.
    pub fn new_frame(frame: CallStackFrame) -> Self {
        Self::Frame(Box::new(frame))
    }
}
