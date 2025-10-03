use context::{ContextTr, FrameStack};
use handler::{
    evm::{ContextDbError, FrameInitResult, FrameTr},
    instructions::InstructionProvider,
    EthFrame, EvmTr, FrameInitOrResult, FrameResult, ItemOrResult,
};
use interpreter::{interpreter::EthInterpreter, interpreter_action::FrameInit, InterpreterTypes};

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

    /// Returns a tuple of mutable references to the context, the inspector, the frame and the instructions.
    ///
    /// This is one of two functions that need to be implemented for Evm. Second one is `all_mut`.
    #[allow(clippy::type_complexity)]
    fn all_inspector(
        &self,
    ) -> (
        &Self::Context,
        &Self::Instructions,
        &Self::Precompiles,
        &FrameStack<Self::Frame>,
        &Self::Inspector,
    );

    /// Returns a tuple of mutable references to the context, the inspector, the frame and the instructions.
    ///
    /// This is one of two functions that need to be implemented for Evm. Second one is `all`.
    #[allow(clippy::type_complexity)]
    fn all_mut_inspector(
        &mut self,
    ) -> (
        &mut Self::Context,
        &mut Self::Instructions,
        &mut Self::Precompiles,
        &mut FrameStack<Self::Frame>,
        &mut Self::Inspector,
    );

    /// Returns a mutable reference to the inspector.
    fn inspector(&mut self) -> &mut Self::Inspector {
        let (_, _, _, _, inspector) = self.all_mut_inspector();
        inspector
    }

    /// Returns a tuple of mutable references to the context and the inspector.
    ///
    /// Useful when you want to allow inspector to modify the context.
    fn ctx_inspector(&mut self) -> (&mut Self::Context, &mut Self::Inspector) {
        let (ctx, _, _, _, inspector) = self.all_mut_inspector();
        (ctx, inspector)
    }

    /// Returns a tuple of mutable references to the context, the inspector and the frame.
    ///
    /// Useful when you want to allow inspector to modify the context and the frame.
    fn ctx_inspector_frame(
        &mut self,
    ) -> (&mut Self::Context, &mut Self::Inspector, &mut Self::Frame) {
        let (ctx, _, _, frame, inspector) = self.all_mut_inspector();
        (ctx, inspector, frame.get())
    }

    /// Returns a tuple of mutable references to the context, the inspector, the frame and the instructions.
    fn ctx_inspector_frame_instructions(
        &mut self,
    ) -> (
        &mut Self::Context,
        &mut Self::Inspector,
        &mut Self::Frame,
        &mut Self::Instructions,
    ) {
        let (ctx, instructions, _, frame, inspector) = self.all_mut_inspector();
        (ctx, inspector, frame.get(), instructions)
    }

    /// Initializes the frame for the given frame input. Frame is pushed to the frame stack.
    #[inline]
    fn inspect_frame_init(
        &mut self,
        mut frame_init: <Self::Frame as FrameTr>::FrameInit,
    ) -> Result<FrameInitResult<'_, Self::Frame>, ContextDbError<Self::Context>> {
        let (ctx, inspector) = self.ctx_inspector();
        if let Some(mut output) = frame_start(ctx, inspector, &mut frame_init.frame_input) {
            frame_end(ctx, inspector, &frame_init.frame_input, &mut output);
            return Ok(ItemOrResult::Result(output));
        }

        let frame_input = frame_init.frame_input.clone();
        if let ItemOrResult::Result(mut output) = self.frame_init(frame_init)? {
            let (ctx, inspector) = self.ctx_inspector();
            frame_end(ctx, inspector, &frame_input, &mut output);
            return Ok(ItemOrResult::Result(output));
        }

        // if it is new frame, initialize the interpreter.
        let (ctx, inspector, frame) = self.ctx_inspector_frame();
        if let Some(frame) = frame.eth_frame() {
            let interp = &mut frame.interpreter;
            inspector.initialize_interp(interp, ctx);
        };
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

        let Some(frame) = frame.eth_frame() else {
            return self.frame_run();
        };

        let next_action = inspect_instructions(
            ctx,
            &mut frame.interpreter,
            inspector,
            instructions.instruction_table(),
        );
        let mut result = frame.process_next_action(ctx, next_action);

        if let Ok(ItemOrResult::Result(frame_result)) = &mut result {
            let (ctx, inspector, frame) = self.ctx_inspector_frame();
            // TODO When all_mut fn is added we can fetch inspector at the top of the function.s
            if let Some(frame) = frame.eth_frame() {
                frame_end(ctx, inspector, &frame.input, frame_result);
                frame.set_finished(true);
            }
        };
        result
    }
}

/// Trait that extends the [`FrameTr`] trait with additional functionality that is needed for inspection.
pub trait InspectorFrame: FrameTr<FrameResult = FrameResult, FrameInit = FrameInit> {
    /// The interpreter types used by this frame.
    type IT: InterpreterTypes;

    /// Returns a mutable reference to the EthFrame.
    ///
    /// If this frame does not have support for tracing (does not contain
    /// the EthFrame) Inspector calls for this frame will be skipped.
    fn eth_frame(&mut self) -> Option<&mut EthFrame<EthInterpreter>>;
}

/// Impl InspectorFrame for EthFrame.
impl InspectorFrame for EthFrame<EthInterpreter> {
    type IT = EthInterpreter;

    fn eth_frame(&mut self) -> Option<&mut EthFrame<EthInterpreter>> {
        Some(self)
    }
}
