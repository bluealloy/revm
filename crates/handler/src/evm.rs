use crate::{
    frame::EthFrameInner, instructions::InstructionProvider,
    item_or_result::NewFrameTrInitOrResult, ItemOrResult, PrecompileProvider,
};
use auto_impl::auto_impl;
use context::{ContextTr, Database, Evm, FrameResult, FrameStack};
use context_interface::context::ContextError;
use interpreter::{interpreter::EthInterpreter, interpreter_action::FrameInit, InterpreterResult};

/// Type alias for database error within a context
pub type ContextDbError<CTX> = ContextError<<<CTX as ContextTr>::Db as Database>::Error>;

/// Type alias for frame init result
pub type FrameInitResult<'a, F> = ItemOrResult<&'a mut F, <F as NewFrameTr>::FrameResult>;

#[auto_impl(&mut, Box)]
pub trait NewFrameTr {
    type FrameResult: Into<FrameResult>;
    type FrameInit: Into<FrameInit>;
}

/// A trait that integrates context, instruction set, and precompiles to create an EVM struct.
///
/// In addition to execution capabilities, this trait provides getter methods for its component fields.
#[auto_impl(&mut, Box)]
pub trait EvmTr {
    /// The context type that implements ContextTr to provide access to execution state
    type Context: ContextTr;
    /// The instruction set type that implements InstructionProvider to define available operations
    type Instructions: InstructionProvider;
    /// The type containing the available precompiled contracts
    type Precompiles: PrecompileProvider<Self::Context>;
    /// The type containing the frame
    type Frame: NewFrameTr;

    /// Returns a mutable reference to the execution context
    fn ctx(&mut self) -> &mut Self::Context;

    /// Returns a mutable reference to the execution context
    fn ctx_mut(&mut self) -> &mut Self::Context {
        self.ctx()
    }

    /// Returns an immutable reference to the execution context
    fn ctx_ref(&self) -> &Self::Context;

    /// Returns mutable references to both the context and instruction set.
    /// This enables atomic access to both components when needed.
    fn ctx_instructions(&mut self) -> (&mut Self::Context, &mut Self::Instructions);

    /// Returns mutable references to both the context and precompiles.
    /// This enables atomic access to both components when needed.
    fn ctx_precompiles(&mut self) -> (&mut Self::Context, &mut Self::Precompiles);

    /// Returns a mutable reference to the frame stack.
    fn frame_stack(&mut self) -> &mut FrameStack<Self::Frame>;

    /// Initializes the frame for the given frame input. Frame is pushed to the frame stack.
    fn frame_init(
        &mut self,
        frame_input: <Self::Frame as NewFrameTr>::FrameInit,
    ) -> Result<FrameInitResult<'_, Self::Frame>, ContextDbError<Self::Context>>;

    /// Rust the frame from the top of the stack. Returns the frame init or result.
    fn frame_run(
        &mut self,
    ) -> Result<NewFrameTrInitOrResult<Self::Frame>, ContextDbError<Self::Context>>;

    /// Returns the result of the frame to the caller. Frame is popped from the frame stack.
    /// Consumes the frame result or returns it if there is more frames to run.
    fn frame_return_result(
        &mut self,
        result: <Self::Frame as NewFrameTr>::FrameResult,
    ) -> Result<Option<<Self::Frame as NewFrameTr>::FrameResult>, ContextDbError<Self::Context>>;
}

impl<CTX, INSP, I, P> EvmTr for Evm<CTX, INSP, I, P, EthFrameInner<EthInterpreter>>
where
    CTX: ContextTr,
    I: InstructionProvider<Context = CTX, InterpreterTypes = EthInterpreter>,
    P: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    type Context = CTX;
    type Instructions = I;
    type Precompiles = P;
    // hardcoded to eth frame.
    type Frame = EthFrameInner<EthInterpreter>;

    #[inline]
    fn ctx(&mut self) -> &mut Self::Context {
        &mut self.ctx
    }

    #[inline]
    fn ctx_ref(&self) -> &Self::Context {
        &self.ctx
    }

    #[inline]
    fn frame_stack(&mut self) -> &mut FrameStack<Self::Frame> {
        &mut self.frame_stack
    }

    /// Initializes the frame for the given frame input. Frame is pushed to the frame stack.
    #[inline]
    fn frame_init(
        &mut self,
        frame_input: <Self::Frame as NewFrameTr>::FrameInit,
    ) -> Result<FrameInitResult<'_, Self::Frame>, ContextDbError<CTX>> {
        let is_first_init = self.frame_stack.index().is_none();
        let new_frame = if is_first_init {
            self.frame_stack.start_init()
        } else {
            self.frame_stack.get_next()
        };
        // TODO
        let ctx = &mut self.ctx;
        let precompiles = &mut self.precompiles;
        let res = EthFrameInner::<EthInterpreter>::init_with_context(
            new_frame,
            ctx,
            precompiles,
            frame_input,
        )?;

        Ok(res.map_frame(|token| {
            if is_first_init {
                self.frame_stack.end_init(token);
            } else {
                self.frame_stack.push(token);
            }
            self.frame_stack.get()
        }))
    }

    /// Rust the frame from the top of the stack. Returns the frame init or result.
    #[inline]
    fn frame_run(&mut self) -> Result<NewFrameTrInitOrResult<Self::Frame>, ContextDbError<CTX>> {
        let frame = self.frame_stack.get();
        let context = &mut self.ctx;
        let instructions = &mut self.instruction;
        let action = frame
            .interpreter
            .run_plain(instructions.instruction_table(), context);
        frame.process_next_action(context, action).inspect(|i| {
            if i.is_result() {
                frame.is_finished = true;
            }
        })
    }

    /// Returns the result of the frame to the caller. Frame is popped from the frame stack.
    #[inline]
    fn frame_return_result(
        &mut self,
        result: <Self::Frame as NewFrameTr>::FrameResult,
    ) -> Result<Option<<Self::Frame as NewFrameTr>::FrameResult>, ContextDbError<Self::Context>>
    {
        if self.frame_stack.get().is_finished {
            self.frame_stack.pop();
        }
        if self.frame_stack.index().is_none() {
            return Ok(Some(result));
        }
        self.frame_stack
            .get()
            .return_result::<_, ContextDbError<Self::Context>>(&mut self.ctx, result)?;
        Ok(None)
    }

    #[inline]
    fn ctx_instructions(&mut self) -> (&mut Self::Context, &mut Self::Instructions) {
        (&mut self.ctx, &mut self.instruction)
    }

    #[inline]
    fn ctx_precompiles(&mut self) -> (&mut Self::Context, &mut Self::Precompiles) {
        (&mut self.ctx, &mut self.precompiles)
    }
}
