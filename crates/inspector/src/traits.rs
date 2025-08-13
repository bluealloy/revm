use crate::{Inspector, JournalExt};
use context::ContextTr;
use handler::{
    evm::{ContextDbError, FrameInitResult, FrameTr},
    instructions::InstructionProvider,
    EthFrame,
    EvmTr,
    FrameInitOrResult,
};
use interpreter::{interpreter::EthInterpreter, FrameInput, Interpreter, InterpreterTypes};
use std::fmt::Debug;

/// Inspector EVM trait. Extends the [`EvmTr`] trait with inspector related methods.
///
/// It contains execution of interpreter with [`crate::Inspector`] calls [`crate::Inspector::step`] and [`crate::Inspector::step_end`] calls.
///
/// It is used inside [`crate::InspectorHandler`] to extend evm with support for inspection.
pub trait InspectorEvmTr:
    EvmTr<
    Frame: InspectorFrame,
    Instructions: InstructionProvider<InterpreterTypes = EthInterpreter, Context = Self::Context>,
    Context: ContextTr<Journal: JournalExt>,
>
{
    /// The inspector type used for EVM execution inspection.
    type Inspector: Inspector<Self::Context, EthInterpreter>;

    /// Returns a mutable reference to the inspector.
    fn inspector(&mut self) -> &mut Self::Inspector;

    /// Returns a tuple of mutable references to the context and the inspector.
    ///
    /// Useful when you want to allow inspector to modify the context.
    fn ctx_inspector(&mut self) -> (&mut Self::Context, &mut Self::Inspector);

    /// Returns a tuple of mutable references to the context, the inspector and the frame.
    ///
    /// Useful when you want to allow inspector to modify the context and the frame.
    fn ctx_inspector_frame(
        &mut self,
    ) -> (&mut Self::Context, &mut Self::Inspector, &mut Self::Frame);

    /// Returns a tuple of mutable references to the context, the inspector, the frame and the instructions.
    fn ctx_inspector_frame_instructions(
        &mut self,
    ) -> (
        &mut Self::Context,
        &mut Self::Inspector,
        &mut Self::Frame,
        &mut Self::Instructions,
    );

    /// Initializes the frame for the given frame input. Frame is pushed to the frame stack.
    fn inspect_frame_init(
        &mut self,
        frame_init: <Self::Frame as FrameTr>::FrameInit,
    ) -> Result<FrameInitResult<'_, Self::Frame>, ContextDbError<Self::Context>>;

    /// Run the frame from the top of the stack. Returns the frame init or result.
    ///
    /// If frame has returned result it would mark it as finished.
    fn inspect_frame_run(
        &mut self,
    ) -> Result<FrameInitOrResult<Self::Frame>, ContextDbError<Self::Context>>;
}

/// Trait that extends the [`FrameTr`] trait with additional functionality that is needed for inspection.
pub trait InspectorFrame: FrameTr {
    /// The interpreter types used by this frame.
    type IT: InterpreterTypes;

    /// Returns a mutable reference to the interpreter.
    fn interpreter(&mut self) -> &mut Interpreter<Self::IT>;

    /// Returns a reference to the frame input. Frame input is needed for call/create/eofcreate [`crate::Inspector`] methods
    fn frame_input(&self) -> &FrameInput;
}

/// Impl InspectorFrame for EthFrame.
impl<EXT: Clone + Debug> InspectorFrame for EthFrame<EthInterpreter, EXT> {
    type IT = EthInterpreter;

    fn interpreter(&mut self) -> &mut Interpreter<Self::IT> {
        &mut self.interpreter
    }

    fn frame_input(&self) -> &FrameInput {
        &self.input
    }
}
