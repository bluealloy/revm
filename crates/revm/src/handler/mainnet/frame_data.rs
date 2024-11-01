use core::ops::Range;
use interpreter::{
    CallOutcome, CreateOutcome, Gas, InstructionResult, InterpreterResult, InterpreterWire,
    NewInterpreter,
};
use primitives::Address;
use std::boxed::Box;
use wiring::{journaled_state::JournalCheckpoint, result::Output};

/// Call CallStackFrame.
//#[derive(Debug)]
//#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CallFrame<W: InterpreterWire> {
    /// Call frame has return memory range where output will be stored.
    pub return_memory_range: Range<usize>,
    /// Journal checkpoint.
    pub checkpoint: JournalCheckpoint,
    /// Interpreter.
    pub interpreter: NewInterpreter<W>,
}

//#[derive(Debug)]
//#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CreateFrame<W: InterpreterWire> {
    /// Create frame has a created address.
    pub created_address: Address,
    /// Journal checkpoint.
    pub checkpoint: JournalCheckpoint,
    /// Interpreter.
    pub interpreter: NewInterpreter<W>,
}

/// Eof Create Frame.
//#[derive(Debug)]
//#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EOFCreateFrame<W: InterpreterWire> {
    pub created_address: Address,
    /// Journal checkpoint.
    pub checkpoint: JournalCheckpoint,
    /// Interpreter.
    pub interpreter: NewInterpreter<W>,
}

/// Call stack frame.
//#[derive(Debug)]
//#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FrameData<W: InterpreterWire> {
    Call(Box<CallFrame<W>>),
    Create(Box<CreateFrame<W>>),
    EOFCreate(Box<EOFCreateFrame<W>>),
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug)]
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

impl<W: InterpreterWire> FrameData<W> {
    pub fn new_create(
        created_address: Address,
        checkpoint: JournalCheckpoint,
        interpreter: NewInterpreter<W>,
    ) -> Self {
        Self::Create(Box::new(CreateFrame {
            created_address,
            checkpoint,
            interpreter,
        }))
    }

    pub fn new_call(
        return_memory_range: Range<usize>,
        checkpoint: JournalCheckpoint,
        interpreter: NewInterpreter<W>,
    ) -> Self {
        Self::Call(Box::new(CallFrame {
            return_memory_range,
            checkpoint,
            interpreter,
        }))
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

    /// Takes frame and returns frame data.
    pub fn into_interpreter(self) -> NewInterpreter<W> {
        match self {
            Self::Call(call_frame) => call_frame.interpreter,
            Self::Create(create_frame) => create_frame.interpreter,
            Self::EOFCreate(eof_create_frame) => eof_create_frame.interpreter,
        }
    }

    /// Returns reference to frame data.
    pub fn interpreter(&self) -> &NewInterpreter<W> {
        match self {
            Self::Call(call_frame) => &call_frame.interpreter,
            Self::Create(create_frame) => &create_frame.interpreter,
            Self::EOFCreate(eof_create_frame) => &eof_create_frame.interpreter,
        }
    }

    /// Returns mutable reference to frame data.
    pub fn interpreter_mut(&mut self) -> &mut NewInterpreter<W> {
        match self {
            Self::Call(call_frame) => &mut call_frame.interpreter,
            Self::Create(create_frame) => &mut create_frame.interpreter,
            Self::EOFCreate(eof_create_frame) => &mut eof_create_frame.interpreter,
        }
    }
}

// impl FrameOrResult {
//     /// Creates new create frame.
//     pub fn new_create_frame(
//         created_address: Address,
//         checkpoint: JournalCheckpoint,
//         interpreter: NewInterpreter<W>,
//     ) -> Self {
//         Self::Frame(Frame::new_create(created_address, checkpoint, interpreter))
//     }

//     pub fn new_eofcreate_frame(
//         created_address: Address,
//         checkpoint: JournalCheckpoint,
//         interpreter: Interpreter,
//     ) -> Self {
//         Self::Frame(Frame::EOFCreate(Box::new(EOFCreateFrame {
//             created_address,
//             frame_data: FrameData {
//                 checkpoint,
//                 interpreter,
//             },
//         })))
//     }

//     /// Creates new call frame.
//     pub fn new_call_frame(
//         return_memory_range: Range<usize>,
//         checkpoint: JournalCheckpoint,
//         interpreter: Interpreter,
//     ) -> Self {
//         Self::Frame(Frame::new_call(
//             return_memory_range,
//             checkpoint,
//             interpreter,
//         ))
//     }

//     /// Creates new create result.
//     pub fn new_create_result(
//         interpreter_result: InterpreterResult,
//         address: Option<Address>,
//     ) -> Self {
//         FrameOrResult::Result(FrameResult::Create(CreateOutcome {
//             result: interpreter_result,
//             address,
//         }))
//     }

//     pub fn new_eofcreate_result(
//         interpreter_result: InterpreterResult,
//         address: Option<Address>,
//     ) -> Self {
//         FrameOrResult::Result(FrameResult::EOFCreate(CreateOutcome {
//             result: interpreter_result,
//             address,
//         }))
//     }

//     pub fn new_call_result(
//         interpreter_result: InterpreterResult,
//         memory_offset: Range<usize>,
//     ) -> Self {
//         FrameOrResult::Result(FrameResult::Call(CallOutcome {
//             result: interpreter_result,
//             memory_offset,
//         }))
//     }
// }
