mod call_inputs;
mod call_outcome;
mod create_inputs;
mod create_outcome;

pub use call_inputs::{CallInput, CallInputs, CallScheme, CallValue};
pub use call_outcome::CallOutcome;
pub use create_inputs::CreateInputs;
pub use create_outcome::CreateOutcome;
use primitives::Bytes;

use crate::{Gas, InstructionResult, InterpreterResult, SharedMemory};
use std::boxed::Box;

/// Input data for creating a new execution frame.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FrameInput {
    /// No input data (empty frame)
    Empty,
    /// `CALL`, `CALLCODE`, `DELEGATECALL`, `STATICCALL` instruction called.
    Call(Box<CallInputs>),
    /// `CREATE` or `CREATE2` instruction called.
    Create(Box<CreateInputs>),
}

/// Initialization data for creating a new execution frame.
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

impl FrameInput {
    /// Reduces the gas limit of the contained Call or Create inputs by `amount`.
    /// Used to charge initial state gas from the frame's regular gas budget
    /// when the reservoir is insufficient.
    pub fn reduce_gas_limit(&mut self, amount: u64) {
        match self {
            FrameInput::Call(inputs) => {
                inputs.gas_limit = inputs.gas_limit.saturating_sub(amount);
            }
            FrameInput::Create(inputs) => {
                inputs.set_gas_limit(inputs.gas_limit().saturating_sub(amount));
            }
            FrameInput::Empty => {}
        }
    }

    /// Returns the state gas reservoir (EIP-8037).
    pub fn reservoir(&self) -> u64 {
        match self {
            FrameInput::Call(inputs) => inputs.reservoir,
            FrameInput::Create(inputs) => inputs.reservoir(),
            FrameInput::Empty => 0,
        }
    }

    /// Sets the state gas reservoir (EIP-8037).
    pub fn set_reservoir(&mut self, reservoir: u64) {
        match self {
            FrameInput::Call(inputs) => inputs.reservoir = reservoir,
            FrameInput::Create(inputs) => inputs.set_reservoir(reservoir),
            FrameInput::Empty => {}
        }
    }
}

impl AsMut<Self> for FrameInput {
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

/// Actions that the interpreter can request from the host environment.
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
    #[inline]
    pub const fn is_call(&self) -> bool {
        matches!(self, InterpreterAction::NewFrame(FrameInput::Call(..)))
    }

    /// Returns `true` if action is create.
    #[inline]
    pub const fn is_create(&self) -> bool {
        matches!(self, InterpreterAction::NewFrame(FrameInput::Create(..)))
    }

    /// Returns `true` if action is return.
    #[inline]
    pub const fn is_return(&self) -> bool {
        matches!(self, InterpreterAction::Return { .. })
    }

    /// Returns [`Gas`] if action is return.
    #[inline]
    pub const fn gas_mut(&mut self) -> Option<&mut Gas> {
        match self {
            InterpreterAction::Return(result) => Some(&mut result.gas),
            _ => None,
        }
    }

    /// Returns [`InterpreterResult`] if action is return.
    ///
    /// Else it returns [None].
    #[inline]
    pub fn into_result_return(self) -> Option<InterpreterResult> {
        match self {
            InterpreterAction::Return(result) => Some(result),
            _ => None,
        }
    }

    /// Returns [`InstructionResult`] if action is return.
    ///
    /// Else it returns [None].
    #[inline]
    pub const fn instruction_result(&self) -> Option<InstructionResult> {
        match self {
            InterpreterAction::Return(result) => Some(result.result),
            _ => None,
        }
    }

    /// Create new frame action with the given frame input.
    #[inline]
    pub const fn new_frame(frame_input: FrameInput) -> Self {
        Self::NewFrame(frame_input)
    }

    /// Create new halt action with the given result and gas.
    #[inline]
    pub const fn new_halt(result: InstructionResult, gas: Gas) -> Self {
        Self::Return(InterpreterResult::new(result, Bytes::new(), gas))
    }

    /// Create new return action with the given result, output and gas.
    #[inline]
    pub const fn new_return(result: InstructionResult, output: Bytes, gas: Gas) -> Self {
        Self::Return(InterpreterResult::new(result, output, gas))
    }

    /// Create new stop action.
    #[inline]
    pub const fn new_stop() -> Self {
        Self::Return(InterpreterResult::new(
            InstructionResult::Stop,
            Bytes::new(),
            Gas::new(0),
        ))
    }
}
