mod call_inputs;
mod call_outcome;
mod create_inputs;
mod create_outcome;
mod eof_create_inputs;

pub use call_inputs::{CallInput, CallInputs, CallScheme, CallValue};
pub use call_outcome::CallOutcome;
pub use create_inputs::CreateInputs;
pub use create_outcome::CreateOutcome;
pub use eof_create_inputs::{EOFCreateInputs, EOFCreateKind};
use primitives::Bytes;

use crate::{Gas, InstructionResult, InterpreterResult, SharedMemory};
use std::boxed::Box;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FrameInput {
    Empty,
    /// `CALL`, `CALLCODE`, `DELEGATECALL`, `STATICCALL`
    /// or EOF `EXTCALL`, `EXTDELEGATECALL`, `EXTSTATICCALL` instruction called.
    Call(Box<CallInputs>),
    /// `CREATE` or `CREATE2` instruction called.
    Create(Box<CreateInputs>),
    /// EOF `CREATE` instruction called.
    EOFCreate(Box<EOFCreateInputs>),
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FrameInit {
    /// depth of the next frame
    pub depth: usize,
    /// shared memory set to this shared context
    pub memory: SharedMemory,
    /// Data needed as input for Interpreter.
    pub frame_input: FrameInput,
}

impl AsMut<Self> for FrameInput {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InterpreterAction {
    /// New frame
    NewFrame(FrameInput),
    /// Interpreter finished execution.
    Return(InterpreterResult),
}

impl InterpreterAction {
    /// Returns `true` if action is call.
    pub fn is_call(&self) -> bool {
        matches!(self, InterpreterAction::NewFrame(FrameInput::Call(..)))
    }

    /// Returns `true` if action is create.
    pub fn is_create(&self) -> bool {
        matches!(self, InterpreterAction::NewFrame(FrameInput::Create(..)))
    }

    /// Returns `true` if action is return.
    pub fn is_return(&self) -> bool {
        matches!(self, InterpreterAction::Return { .. })
    }

    /// Returns [`InterpreterResult`] if action is return.
    ///
    /// Else it returns [None].
    pub fn into_result_return(self) -> Option<InterpreterResult> {
        match self {
            InterpreterAction::Return(result) => Some(result),
            _ => None,
        }
    }

    /// Returns [`InstructionResult`] if action is return.
    ///
    /// Else it returns [None].
    pub fn instruction_result(&self) -> Option<InstructionResult> {
        match self {
            InterpreterAction::Return(result) => Some(result.result),
            _ => None,
        }
    }

    /// Create new frame action with the given frame input.
    pub fn new_frame(frame_input: FrameInput) -> Self {
        Self::NewFrame(frame_input)
    }

    /// Create new halt action with the given result and gas.
    pub fn new_halt(result: InstructionResult, gas: Gas) -> Self {
        Self::Return(InterpreterResult::new(result, Bytes::new(), gas))
    }

    /// Create new return action with the given result, output and gas.
    pub fn new_return(result: InstructionResult, output: Bytes, gas: Gas) -> Self {
        Self::Return(InterpreterResult::new(result, output, gas))
    }

    /// Create new stop action.
    pub fn new_stop() -> Self {
        Self::Return(InterpreterResult::new(
            InstructionResult::Stop,
            Bytes::new(),
            Gas::new(0),
        ))
    }
}
