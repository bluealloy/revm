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
    Frame: InspectorFrameTr<IT = EthInterpreter>,
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
        &Self::Inspector,
        &Self::Instructions,
        &FrameStack<Self::Frame>,
    );

    /// Returns a tuple of mutable references to the context, the inspector, the frame and the instructions.
    ///
    /// This is one of two functions that need to be implemented for Evm. Second one is `all`.
    #[allow(clippy::type_complexity)]
    fn all_mut_inspector(
        &mut self,
    ) -> (
        &mut Self::Context,
        &mut Self::Inspector,
        &mut FrameStack<Self::Frame>,
        &mut Self::Instructions,
    );

    /// Returns a mutable reference to the inspector.
    fn inspector(&mut self) -> &mut Self::Inspector {
        self.all_mut_inspector().1
    }

    /// Returns a tuple of mutable references to the context and the inspector.
    ///
    /// Useful when you want to allow inspector to modify the context.
    fn ctx_inspector(&mut self) -> (&mut Self::Context, &mut Self::Inspector) {
        let (ctx, inspector, _, _) = self.all_mut_inspector();
        (ctx, inspector)
    }

    /// Returns a tuple of mutable references to the context, the inspector and the frame.
    ///
    /// Useful when you want to allow inspector to modify the context and the frame.
    fn ctx_inspector_frame(
        &mut self,
    ) -> (&mut Self::Context, &mut Self::Inspector, &mut Self::Frame) {
        let (ctx, inspector, frame, _) = self.all_mut_inspector();
        (ctx, inspector, frame.get_mut())
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
        let (ctx, inspector, frame, instructions) = self.all_mut_inspector();
        (ctx, inspector, frame.get_mut(), instructions)
    }

    /// Initializes the frame for the given frame input. Frame is pushed to the frame stack.
    #[inline]
    fn inspect_frame_init(
        &mut self,
        frame_init: <Self::Frame as FrameTr>::FrameInit,
    ) -> Result<FrameInitResult<'_, Self::Frame>, ContextDbError<Self::Context>> {
        // TODO try from or skip it if it is not possible.
        // assume that we have from result to Frame::Result if FrameInit is try_from is possible.
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
        if let Some(frame) = &mut frame.eth_frame() {
            inspector.initialize_interp(&mut frame.interpreter, ctx);
        }
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

        if let Some(eth_frame) = frame.eth_frame() {
            let action = inspect_instructions(
                ctx,
                &mut eth_frame.interpreter,
                inspector,
                instructions.instruction_table(),
            );

            let (_, _, cfg, journal, _, _) = ctx.all_mut();

            // process the next action and cast both FrameInit and FrameResult to the frame type.
            Ok(eth_frame
                .process_next_action(cfg, journal, action)
                .map(|frame_init| frame_init.into())
                .map_result(|r| r.into()))
        } else {
            self.frame_run()
        }
    }
}

/// Trait that extends the [`FrameTr`] trait with additional functionality that is needed for inspection.
pub trait InspectorFrameTr:
    FrameTr<
    FrameResult: From<FrameResult> + Into<FrameResult>,
    FrameInit: From<FrameInit> + Into<FrameInit> + Clone,
>
{
    /// The interpreter types used by this frame.
    type IT: InterpreterTypes;

    /// Returns a mutable reference to the inner EthFrame if it exists.
    ///
    /// If EthFrame is not used inside frame following Inspector functions are going to be skipped for this frame:
    /// * [`crate::Inspector::initialize_interp`]
    /// * [`crate::Inspector::step`]
    /// * [`crate::Inspector::step_end`]
    /// * [`crate::Inspector::log`]
    /// * [`crate::Inspector::selfdestruct`]
    fn eth_frame(&mut self) -> Option<&mut EthFrame<Self::IT>>;
}

/// Impl InspectorFrameTr for EthFrame.
impl InspectorFrameTr for EthFrame<EthInterpreter> {
    type IT = EthInterpreter;

    fn eth_frame(&mut self) -> Option<&mut EthFrame<Self::IT>> {
        Some(self)
    }
}
