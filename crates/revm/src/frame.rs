use crate::JournalCheckpoint;
use crate::{interpreter::Interpreter, primitives::Address};
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
