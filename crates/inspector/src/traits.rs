use context::{ContextError, ContextTr, Database};
use handler::{
    evm::NewFrameTr, instructions::InstructionProvider, EthFrameInner, EvmTr, ItemOrResult,
    NewFrameTrInitOrResult,
};
use interpreter::{interpreter::EthInterpreter, FrameInput, Interpreter, InterpreterTypes};

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
    Frame = EthFrameInner<EthInterpreter>,
    Instructions: InstructionProvider<InterpreterTypes = EthInterpreter, Context = Self::Context>,
    Context: ContextTr<Journal: JournalExt>,
>
{
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

    // /// Runs the inspector on the interpreter.
    // ///
    // /// This function is called by the EVM when it needs to inspect the Interpreter loop.
    // /// It is responsible for calling the inspector's methods and instructions from table.
    // fn run_inspect_interpreter(
    //     &mut self,
    //     interpreter: &mut Interpreter<
    //         <Self::Instructions as InstructionProvider>::InterpreterTypes,
    //     >,
    // ) -> <<Self::Instructions as InstructionProvider>::InterpreterTypes as InterpreterTypes>::Output;

    /// Initializes the frame for the given frame input. Frame is pushed to the frame stack.
    #[inline]
    fn inspect_frame_init(
        &mut self,
        mut frame_init: <Self::Frame as NewFrameTr>::FrameInit,
    ) -> Result<
        ItemOrResult<&mut Self::Frame, <Self::Frame as NewFrameTr>::FrameResult>,
        ContextError<<<Self::Context as ContextTr>::Db as Database>::Error>,
    > {
        let (ctx, inspector) = self.ctx_inspector();
        if let Some(mut output) = frame_start(ctx, inspector, &mut frame_init.frame_input) {
            frame_end(ctx, inspector, &frame_init.frame_input, &mut output);
            return Ok(ItemOrResult::Result(output));
        }
        if let ItemOrResult::Result(frame) = self.frame_init(frame_init)? {
            return Ok(ItemOrResult::Result(frame));
        }

        // if it is new frame, initialize the interpreter.
        let (ctx, inspector, frame) = self.ctx_inspector_frame();
        let interp = frame.interpreter();
        inspector.initialize_interp(interp, ctx);
        return Ok(ItemOrResult::Item(frame));
    }

    /// Rust the frame from the top of the stack. Returns the frame init or result.
    #[inline]
    fn inspect_frame_run(
        &mut self,
    ) -> Result<
        NewFrameTrInitOrResult<Self::Frame>,
        ContextError<<<Self::Context as ContextTr>::Db as Database>::Error>,
    > {
        let (ctx, inspector, frame, instructions) = self.ctx_inspector_frame_instructions();

        let next_action = inspect_instructions(
            ctx,
            frame.interpreter(),
            inspector,
            instructions.instruction_table(),
        );
        frame.process_next_action(ctx, next_action)
    }

    /// Returns the result of the frame to the caller. Frame is popped from the frame stack.
    /// Consumes the frame result or returns it if there is more frames to run.
    #[inline]
    fn inspect_frame_return_result(
        &mut self,
        result: <Self::Frame as NewFrameTr>::FrameResult,
    ) -> Result<
        Option<<Self::Frame as NewFrameTr>::FrameResult>,
        ContextError<<<Self::Context as ContextTr>::Db as Database>::Error>,
    > {
        self.frame_return_result(result)
    }
}

/// Traits that extends the Frame with additional functionality that is needed for inspection
///
/// It is implemented for [`EthFrame`] as default Ethereum frame implementation.
pub trait InspectorFrame: NewFrameTr {
    type IT: InterpreterTypes;

    /// Returns a mutable reference to the interpreter.
    fn interpreter(&mut self) -> &mut Interpreter<Self::IT>;

    /// Returns a reference to the frame input. Frame input is needed for call/create/eofcreate [`crate::Inspector`] methods
    fn frame_input(&self) -> &FrameInput;
}

/// Impl InspectorFrame for EthFrame.
impl InspectorFrame for EthFrameInner<EthInterpreter> {
    type IT = EthInterpreter;

    // fn run_inspect(&mut self, evm: &mut Self::Evm) -> Result<FrameInitOrResult<Self>, Self::Error> {
    //     let interpreter = self.interpreter();
    //     let next_action = evm.run_inspect_interpreter(interpreter);
    //     self.process_next_action(evm, next_action)
    // }

    fn interpreter(&mut self) -> &mut Interpreter<Self::IT> {
        &mut self.interpreter
    }

    fn frame_input(&self) -> &FrameInput {
        &self.input
    }
}
