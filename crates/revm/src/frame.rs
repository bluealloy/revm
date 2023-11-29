use crate::{
    interpreter::{Interpreter, InterpreterResult},
    primitives::Address,
    JournalCheckpoint,
};
use core::ops::Range;

/// Call CallStackFrame.
#[derive(Debug)]
pub struct CallStackFrame {
    /// True if it is create false if it is call.
    /// TODO make a enum for this.
    pub is_create: bool,
    /// Journal checkpoint
    pub checkpoint: JournalCheckpoint,
    /// temporary. If it is create it should have address.
    pub created_address: Option<Address>,
    /// temporary. Call range
    pub subcall_return_memory_range: Range<usize>,
    /// Interpreter
    pub interpreter: Interpreter,
}

/// Contains either a frame or a result.
pub enum FrameOrResult {
    /// Boxed stack frame
    Frame(Box<CallStackFrame>),
    /// Interpreter result
    Result(InterpreterResult),
}

impl FrameOrResult {
    /// Returns new frame.
    pub fn new_frame(frame: CallStackFrame) -> Self {
        Self::Frame(Box::new(frame))
    }
}
