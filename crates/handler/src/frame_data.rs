use context_interface::result::Output;
use core::ops::Range;
use interpreter::{CallOutcome, CreateOutcome, Gas, InstructionResult, InterpreterResult};
use primitives::Address;

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

/// Eof Create Frame
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EOFCreateFrame {
    pub created_address: Address,
}

/// Frame Data
///
/// [`FrameData`] bundles different types of frames.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FrameData {
    Call(CallFrame),
    Create(CreateFrame),
    EOFCreate(EOFCreateFrame),
}

/// Frame Result
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Clone)]
pub enum FrameResult {
    Call(CallOutcome),
    Create(CreateOutcome),
    EOFCreate(CreateOutcome),
}

impl FrameResult {
    /// Casts frame result to interpreter result.
    #[inline]
    pub fn into_interpreter_result(self) -> InterpreterResult {
        match self {
            FrameResult::Call(outcome) => outcome.result,
            FrameResult::Create(outcome) => outcome.result,
            FrameResult::EOFCreate(outcome) => outcome.result,
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
            FrameResult::EOFCreate(outcome) => {
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
            FrameResult::EOFCreate(outcome) => &outcome.result.gas,
        }
    }

    /// Returns mutable reference to interpreter result.
    #[inline]
    pub fn gas_mut(&mut self) -> &mut Gas {
        match self {
            FrameResult::Call(outcome) => &mut outcome.result.gas,
            FrameResult::Create(outcome) => &mut outcome.result.gas,
            FrameResult::EOFCreate(outcome) => &mut outcome.result.gas,
        }
    }

    /// Returns reference to interpreter result.
    #[inline]
    pub fn interpreter_result(&self) -> &InterpreterResult {
        match self {
            FrameResult::Call(outcome) => &outcome.result,
            FrameResult::Create(outcome) => &outcome.result,
            FrameResult::EOFCreate(outcome) => &outcome.result,
        }
    }

    /// Returns mutable reference to interpreter result.
    #[inline]
    pub fn interpreter_result_mut(&mut self) -> &InterpreterResult {
        match self {
            FrameResult::Call(outcome) => &mut outcome.result,
            FrameResult::Create(outcome) => &mut outcome.result,
            FrameResult::EOFCreate(outcome) => &mut outcome.result,
        }
    }

    /// Return Instruction result.
    #[inline]
    pub fn instruction_result(&self) -> InstructionResult {
        self.interpreter_result().result
    }
}

impl FrameData {
    pub fn new_create(created_address: Address) -> Self {
        Self::Create(CreateFrame { created_address })
    }

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
