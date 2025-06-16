//! Contains the `[OpEvm]` type and its implementation of the execution EVM traits.
use crate::precompiles::OpPrecompiles;
use revm::{
    context::{ContextError, ContextSetters, Evm, FrameStack},
    context_interface::ContextTr,
    handler::{
        evm::NewFrameTr,
        instructions::{EthInstructions, InstructionProvider},
        EthFrameInner, EvmTr, ItemOrResult, NewFrameTrInitOrResult, PrecompileProvider,
    },
    inspector::{InspectorEvmTr, JournalExt},
    interpreter::{interpreter::EthInterpreter, Interpreter, InterpreterAction, InterpreterResult, InterpreterTypes},
    Database, Inspector,
};

/// Optimism EVM extends the [`Evm`] type with Optimism specific types and logic.
#[derive(Debug, Clone)]
pub struct OpEvm<
    CTX,
    INSP,
    I = EthInstructions<EthInterpreter, CTX>,
    P = OpPrecompiles,
    F = EthFrameInner<EthInterpreter>,
>(
    /// Inner EVM type.
    pub Evm<CTX, INSP, I, P, F>,
);

impl<CTX: ContextTr, INSP> OpEvm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, OpPrecompiles> {
    /// Create a new Optimism EVM.
    pub fn new(ctx: CTX, inspector: INSP) -> Self {
        Self(Evm {
            ctx,
            inspector,
            instruction: EthInstructions::new_mainnet(),
            precompiles: OpPrecompiles::default(),
            frame_stack: FrameStack::new(),
        })
    }
}

impl<CTX, INSP, I, P> OpEvm<CTX, INSP, I, P> {
    /// Consumed self and returns a new Evm type with given Inspector.
    pub fn with_inspector<OINSP>(self, inspector: OINSP) -> OpEvm<CTX, OINSP, I, P> {
        OpEvm(self.0.with_inspector(inspector))
    }

    /// Consumes self and returns a new Evm type with given Precompiles.
    pub fn with_precompiles<OP>(self, precompiles: OP) -> OpEvm<CTX, INSP, I, OP> {
        OpEvm(self.0.with_precompiles(precompiles))
    }

    /// Consumes self and returns the inner Inspector.
    pub fn into_inspector(self) -> INSP {
        self.0.into_inspector()
    }
}

impl<CTX, INSP, I, P> InspectorEvmTr for OpEvm<CTX, INSP, I, P>
where
    CTX: ContextTr<Journal: JournalExt> + ContextSetters,
    I: InstructionProvider<Context = CTX, InterpreterTypes = EthInterpreter>,
    P: PrecompileProvider<CTX, Output = InterpreterResult>,
    INSP: Inspector<CTX, I::InterpreterTypes>,
{
    type Inspector = INSP;

    fn inspector(&mut self) -> &mut Self::Inspector {
        &mut self.0.inspector
    }

    fn ctx_inspector(&mut self) -> (&mut Self::Context, &mut Self::Inspector) {
        (&mut self.0.ctx, &mut self.0.inspector)
    }

    fn ctx_inspector_frame(
        &mut self,
    ) -> (&mut Self::Context, &mut Self::Inspector, &mut Self::Frame) {
        (
            &mut self.0.ctx,
            &mut self.0.inspector,
            self.0.frame_stack.get(),
        )
    }

    fn ctx_inspector_frame_instructions(
        &mut self,
    ) -> (
        &mut Self::Context,
        &mut Self::Inspector,
        &mut Self::Frame,
        &mut Self::Instructions,
    ) {
        (
            &mut self.0.ctx,
            &mut self.0.inspector,
            self.0.frame_stack.get(),
            &mut self.0.instruction,
        )
    }
}

impl<CTX, INSP, I, P> EvmTr for OpEvm<CTX, INSP, I, P, EthFrameInner<EthInterpreter>>
where
    CTX: ContextTr,
    I: InstructionProvider<Context = CTX, InterpreterTypes = EthInterpreter>,
    P: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    type Context = CTX;
    type Instructions = I;
    type Precompiles = P;
    type Frame = EthFrameInner<EthInterpreter>;

    fn ctx(&mut self) -> &mut Self::Context {
        &mut self.0.ctx
    }

    fn ctx_ref(&self) -> &Self::Context {
        &self.0.ctx
    }

    fn ctx_instructions(&mut self) -> (&mut Self::Context, &mut Self::Instructions) {
        (&mut self.0.ctx, &mut self.0.instruction)
    }

    fn ctx_precompiles(&mut self) -> (&mut Self::Context, &mut Self::Precompiles) {
        (&mut self.0.ctx, &mut self.0.precompiles)
    }

    fn frame_init(
        &mut self,
        frame_input: <Self::Frame as NewFrameTr>::FrameInit,
    ) -> Result<
        ItemOrResult<&mut Self::Frame, <Self::Frame as NewFrameTr>::FrameResult>,
        ContextError<<<Self::Context as ContextTr>::Db as Database>::Error>,
    > {
        self.0.frame_init(frame_input)
    }

    fn frame_run(
        &mut self,
    ) -> Result<
        NewFrameTrInitOrResult<Self::Frame>,
        ContextError<<<Self::Context as ContextTr>::Db as Database>::Error>,
    > {
        self.0.frame_run()
    }

    #[doc = " Returns the result of the frame to the caller. Frame is popped from the frame stack."]
    #[doc = " Consumes the frame result or returns it if there is more frames to run."]
    fn frame_return_result(
        &mut self,
        result: <Self::Frame as NewFrameTr>::FrameResult,
    ) -> Result<
        Option<<Self::Frame as NewFrameTr>::FrameResult>,
        ContextError<<<Self::Context as ContextTr>::Db as Database>::Error>,
    > {
        self.0.frame_return_result(result)
    }
}
