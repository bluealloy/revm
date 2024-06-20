mod call_inputs;
mod call_outcome;
mod create_inputs;
mod create_outcome;
mod eof_create_inputs;

pub use call_inputs::{CallInputs, CallScheme, CallValue};
pub use call_outcome::CallOutcome;
pub use create_inputs::{CreateInputs, CreateScheme};
pub use create_outcome::CreateOutcome;
pub use eof_create_inputs::{EOFCreateInputs, EOFCreateKind};

use crate::InterpreterResult;
use std::boxed::Box;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InterpreterAction {
    /// CALL, CALLCODE, DELEGATECALL, STATICCALL
    /// or EOF EXT instuction called.
    Call { inputs: Box<CallInputs> },
    /// CREATE or CREATE2 instruction called.
    Create { inputs: Box<CreateInputs> },
    /// EOF CREATE instruction called.
    EOFCreate { inputs: Box<EOFCreateInputs> },
    /// Interpreter finished execution.
    Return { result: InterpreterResult },
    /// No action
    #[default]
    None,
}

impl InterpreterAction {
    /// Returns true if action is call.
    pub fn is_call(&self) -> bool {
        matches!(self, InterpreterAction::Call { .. })
    }

    /// Returns true if action is create.
    pub fn is_create(&self) -> bool {
        matches!(self, InterpreterAction::Create { .. })
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
