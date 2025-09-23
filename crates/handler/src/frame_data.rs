use context::Cfg;
use context_interface::result::Output;
use core::ops::Range;
use interpreter::{CallOutcome, CreateOutcome, Gas, InstructionResult, InterpreterResult};
use primitives::Address;

use crate::{frame::CheckpointResult, return_create};

/// Call Frame
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CallFrame {
    /// Call frame has return memory range where output will be stored.
    pub return_memory_range: Range<usize>,
}

/// Create Frame
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CreateFrame {
    /// Create frame has a created address.
    pub created_address: Address,
}

/// Frame Data
///
/// [`FrameData`] bundles different types of frames.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FrameData {
    /// Call frame data.
    Call(CallFrame),
    /// Create frame data.
    Create(CreateFrame),
}

impl FrameData {
    /// Processes the next interpreter action, either creating a new frame or returning a result.
    pub fn process_next_action<CFG: Cfg>(
        &self,
        cfg: CFG,
        mut interpreter_result: InterpreterResult,
    ) -> (FrameResult, CheckpointResult) {
        // Handle return from frame
        let result = match self {
            FrameData::Call(frame) => {
                let is_ok = interpreter_result.result.is_ok();
                let res = if is_ok {
                    CheckpointResult::Commit
                } else {
                    CheckpointResult::Revert
                };

                (
                    FrameResult::new_call(interpreter_result, frame.return_memory_range.clone()),
                    res,
                )
            }
            FrameData::Create(frame) => {
                let res = return_create(&mut interpreter_result, frame.created_address, cfg);

                (
                    FrameResult::Create(CreateOutcome::new(
                        interpreter_result,
                        Some(frame.created_address),
                    )),
                    res,
                )
            }
        };

        result
    }
}

/// Frame Result
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum FrameResult {
    /// Call frame result.
    Call(CallOutcome),
    /// Create frame result.
    Create(CreateOutcome),
}

impl FrameResult {
    /// Creates a new call frame result.
    pub fn new_call(result: InterpreterResult, memory_offset: Range<usize>) -> Self {
        Self::Call(CallOutcome {
            result,
            memory_offset,
        })
    }

    /// Creates a new create frame result.
    pub fn new_create(outcome: CreateOutcome) -> Self {
        Self::Create(outcome)
    }

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
    pub fn interpreter_result_mut(&mut self) -> &mut InterpreterResult {
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

impl FrameData {
    /// Creates a new create frame data.
    pub fn new_create(created_address: Address) -> Self {
        Self::Create(CreateFrame { created_address })
    }

    /// Creates a new call frame data.
    pub fn new_call(return_memory_range: Range<usize>) -> Self {
        Self::Call(CallFrame {
            return_memory_range,
        })
    }

    /// Returns true if frame is call frame.
    pub fn is_call(&self) -> bool {
        matches!(self, Self::Call { .. })
    }

    /// Returns true if frame is create frame.
    pub fn is_create(&self) -> bool {
        matches!(self, Self::Create { .. })
    }

    /// Returns created address if frame is create otherwise returns None.
    pub fn created_address(&self) -> Option<Address> {
        match self {
            Self::Create(create_frame) => Some(create_frame.created_address),
            _ => None,
        }
    }
}
