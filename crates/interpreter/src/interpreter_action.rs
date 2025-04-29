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

use crate::InterpreterResult;
use std::boxed::Box;

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FrameInput {
    /// `CALL`, `CALLCODE`, `DELEGATECALL`, `STATICCALL`
    /// or EOF `EXTCALL`, `EXTDELEGATECALL`, `EXTSTATICCALL` instruction called.
    Call(Box<CallInputs>),
    /// `CREATE` or `CREATE2` instruction called.
    Create(Box<CreateInputs>),
    /// EOF `CREATE` instruction called.
    EOFCreate(Box<EOFCreateInputs>),
}

impl AsMut<Self> for FrameInput {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InterpreterAction {
    /// New frame
    NewFrame(FrameInput),
    /// Interpreter finished execution.
    Return { result: InterpreterResult },
    /// No action
    #[default]
    None,
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

    /// Returns `true` if action is none.
    pub fn is_none(&self) -> bool {
        matches!(self, InterpreterAction::None)
    }

    /// Returns `true` if action is some.
    pub fn is_some(&self) -> bool {
        !self.is_none()
    }

    /// Returns [`InterpreterResult`] if action is return.
    ///
    /// Else it returns [None].
    pub fn into_result_return(self) -> Option<InterpreterResult> {
        match self {
            InterpreterAction::Return { result } => Some(result),
            _ => None,
        }
    }
}
