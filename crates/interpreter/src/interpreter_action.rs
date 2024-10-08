mod call_inputs;
mod call_outcome;
mod create_inputs;
mod create_outcome;
mod eof_create_inputs;

pub use call_inputs::{CallInputs, CallScheme, CallValue};
pub use call_outcome::CallOutcome;
pub use create_inputs::CreateInputs;
pub use create_outcome::CreateOutcome;
pub use eof_create_inputs::{EOFCreateInputs, EOFCreateKind};

use crate::InterpreterResult;
use std::boxed::Box;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NewFrameAction {
    /// CALL, CALLCODE, DELEGATECALL, STATICCALL
    /// or EOF EXT*CALL instruction called.
    Call(Box<CallInputs>),
    /// CREATE or CREATE2 instruction called.
    Create(Box<CreateInputs>),
    /// EOF CREATE instruction called.
    EOFCreate(Box<EOFCreateInputs>),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InterpreterAction {
    /// New frame
    NewFrame(NewFrameAction),
    /// Interpreter finished execution.
    Return { result: InterpreterResult },
    /// No action
    #[default]
    None,
}

impl InterpreterAction {
    /// Returns true if action is call.
    pub fn is_call(&self) -> bool {
        matches!(self, InterpreterAction::NewFrame(NewFrameAction::Call(..)))
    }

    /// Returns true if action is create.
    pub fn is_create(&self) -> bool {
        matches!(
            self,
            InterpreterAction::NewFrame(NewFrameAction::Create(..))
        )
    }

    /// Returns true if action is return.
    pub fn is_return(&self) -> bool {
        matches!(self, InterpreterAction::Return { .. })
    }

    /// Returns true if action is none.
    pub fn is_none(&self) -> bool {
        matches!(self, InterpreterAction::None)
    }

    /// Returns true if action is some.
    pub fn is_some(&self) -> bool {
        !self.is_none()
    }

    /// Returns result if action is return.
    pub fn into_result_return(self) -> Option<InterpreterResult> {
        match self {
            InterpreterAction::Return { result } => Some(result),
            _ => None,
        }
    }
}
