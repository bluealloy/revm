use context::ContextTr;
use handler::{
    evm::{ContextDbError, FrameInitResult, FrameTr},
    instructions::InstructionProvider,
    EthFrame, EvmTr, FrameInitOrResult, FrameResult, ItemOrResult,
};
use interpreter::{
    interpreter::EthInterpreter, FrameInput, Interpreter, InterpreterAction, InterpreterTypes,
};

use crate::{
    handler::{frame_end, frame_start},
    inspect_instructions, Inspector, JournalExt,
};

/// Inspector EVM trait. Extends the [`EvmTr`] trait with inspector related methods.
///
/// It contains execution of interpreter with [`crate::Inspector`] calls [`crate::Inspector::step`] and [`crate::Inspector::step_end`] calls.
///
/// It is used inside [`crate::InspectorHandler`] to extend evm with support for inspection.
pub trait InspectorEvmTr:
    EvmTr<
    Frame: InspectorFrame<IT = EthInterpreter>,
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
    #[inline]
    fn inspect_frame_init(
        &mut self,
        frame_init: <Self::Frame as FrameTr>::FrameInit,
    ) -> Result<FrameInitResult<'_, Self::Frame>, ContextDbError<Self::Context>> {
        let mut eth_frame_init = frame_init.clone().into();
        let (ctx, inspector) = self.ctx_inspector();
        if let Some(mut output) = frame_start(ctx, inspector, &mut eth_frame_init.frame_input) {
            frame_end(ctx, inspector, &eth_frame_init.frame_input, &mut output);
            return Ok(ItemOrResult::Result(output.into()));
        }

        if let ItemOrResult::Result(output) = self.frame_init(frame_init)? {
            let (ctx, inspector) = self.ctx_inspector();
            let mut output = output.into();
            frame_end(ctx, inspector, &eth_frame_init.frame_input, &mut output);
            return Ok(ItemOrResult::Result(output.into()));
        }

        // if it is new frame, initialize the interpreter.
        let (ctx, inspector, frame) = self.ctx_inspector_frame();
        let interp = frame.interpreter();
        inspector.initialize_interp(interp, ctx);
        Ok(ItemOrResult::Item(frame))
    }

    /// Run the frame from the top of the stack. Returns the frame init or result.
    ///
    /// If frame has returned result it would mark it as finished.
    #[inline]
    fn inspect_frame_run(
        &mut self,
    ) -> Result<FrameInitOrResult<Self::Frame>, ContextDbError<Self::Context>> {
        let (ctx, inspector, frame, instructions) = self.ctx_inspector_frame_instructions();

        let next_action = inspect_instructions(
            ctx,
            frame.interpreter(),
            inspector,
            instructions.instruction_table(),
        );
        let mut result = frame.process_next_action(ctx, next_action);

        if let Ok(ItemOrResult::Result(frame_result)) = &mut result {
            let (ctx, inspector, frame) = self.ctx_inspector_frame();
            frame_end(ctx, inspector, frame.frame_input(), frame_result);
            frame.set_finished(true);
        };
        result
    }
}

/// Trait that extends the [`FrameTr`] trait with additional functionality that is needed for inspection.
pub trait InspectorFrame: FrameTr {
    /// The interpreter types used by this frame.
    type IT: InterpreterTypes;

    /// Returns a mutable reference to the interpreter.
    fn interpreter(&mut self) -> &mut Interpreter<Self::IT>;

    /// Returns a reference to the frame input. Frame input is needed for call/create/eofcreate [`crate::Inspector`] methods
    fn frame_input(&self) -> &FrameInput;

    // fn process_next_action(
    //     &mut self,
    //     ctx: &mut Context,
    //     action: InterpreterAction,
    // ) -> FrameInitOrResult<Self>;
}

/// Impl InspectorFrame for EthFrame.
impl InspectorFrame for EthFrame<EthInterpreter> {
    type IT = EthInterpreter;

    fn interpreter(&mut self) -> &mut Interpreter<Self::IT> {
        &mut self.interpreter
    }

    fn frame_input(&self) -> &FrameInput {
        &self.input
    }

    // fn process_next_action(
    //     &mut self,
    //     ctx: &mut Context,
    //     action: InterpreterAction,
    // ) -> FrameInitOrResult<Self> {
    //     self.process_next_action(ctx, action)
    // }
}
