// MonadEvm - wrapper around base Evm with Monad-specific types.
use crate::{
    instructions::{monad_instructions, MonadInstructions},
    precompiles::MonadPrecompiles,
    MonadSpecId,
};
use revm::{
    context::{Cfg, ContextError, ContextSetters, Evm, FrameStack},
    context_interface::ContextTr,
    handler::{
        evm::FrameTr, instructions::InstructionProvider, EthFrame, EvmTr, FrameInitOrResult,
        ItemOrResult, PrecompileProvider,
    },
    inspector::{InspectorEvmTr, JournalExt},
    interpreter::{interpreter::EthInterpreter, InterpreterResult},
    Database, Inspector,
};

/// Monad EVM with custom gas costs and precompiles.
#[derive(Debug, Clone)]
pub struct MonadEvm<
    CTX,
    INSP,
    I = MonadInstructions<CTX>,
    P = MonadPrecompiles,
    F = EthFrame<EthInterpreter>,
>(pub Evm<CTX, INSP, I, P, F>);

impl<CTX, INSP> MonadEvm<CTX, INSP, MonadInstructions<CTX>, MonadPrecompiles>
where
    CTX: ContextTr<Cfg: Cfg<Spec = MonadSpecId>>,
{
    /// Create a new Monad EVM with custom gas costs and precompiles.
    pub fn new(ctx: CTX, inspector: INSP) -> Self {
        let spec = ctx.cfg().spec();
        Self(Evm {
            ctx,
            inspector,
            instruction: monad_instructions(spec),
            precompiles: MonadPrecompiles::new_with_spec(spec),
            frame_stack: FrameStack::new_prealloc(8),
        })
    }
}

impl<CTX, INSP, I, P> MonadEvm<CTX, INSP, I, P> {
    /// Consume self and return a new EVM with given Inspector.
    pub fn with_inspector<OINSP>(self, inspector: OINSP) -> MonadEvm<CTX, OINSP, I, P> {
        MonadEvm(self.0.with_inspector(inspector))
    }

    /// Consume self and return a new EVM with given Precompiles.
    pub fn with_precompiles<OP>(self, precompiles: OP) -> MonadEvm<CTX, INSP, I, OP> {
        MonadEvm(self.0.with_precompiles(precompiles))
    }

    /// Consume self and return the inner Inspector.
    pub fn into_inspector(self) -> INSP {
        self.0.into_inspector()
    }
}

impl<CTX, INSP, I, P> InspectorEvmTr for MonadEvm<CTX, INSP, I, P>
where
    CTX: ContextTr<Journal: JournalExt> + ContextSetters,
    I: InstructionProvider<Context = CTX, InterpreterTypes = EthInterpreter>,
    P: PrecompileProvider<CTX, Output = InterpreterResult>,
    INSP: Inspector<CTX, I::InterpreterTypes>,
{
    type Inspector = INSP;

    #[inline]
    fn all_inspector(
        &self,
    ) -> (
        &Self::Context,
        &Self::Instructions,
        &Self::Precompiles,
        &FrameStack<Self::Frame>,
        &Self::Inspector,
    ) {
        self.0.all_inspector()
    }

    #[inline]
    fn all_mut_inspector(
        &mut self,
    ) -> (
        &mut Self::Context,
        &mut Self::Instructions,
        &mut Self::Precompiles,
        &mut FrameStack<Self::Frame>,
        &mut Self::Inspector,
    ) {
        self.0.all_mut_inspector()
    }
}

impl<CTX, INSP, I, P> EvmTr for MonadEvm<CTX, INSP, I, P, EthFrame<EthInterpreter>>
where
    CTX: ContextTr,
    I: InstructionProvider<Context = CTX, InterpreterTypes = EthInterpreter>,
    P: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    type Context = CTX;
    type Instructions = I;
    type Precompiles = P;
    type Frame = EthFrame<EthInterpreter>;

    #[inline]
    fn all(
        &self,
    ) -> (
        &Self::Context,
        &Self::Instructions,
        &Self::Precompiles,
        &FrameStack<Self::Frame>,
    ) {
        self.0.all()
    }

    #[inline]
    fn all_mut(
        &mut self,
    ) -> (
        &mut Self::Context,
        &mut Self::Instructions,
        &mut Self::Precompiles,
        &mut FrameStack<Self::Frame>,
    ) {
        self.0.all_mut()
    }

    fn frame_init(
        &mut self,
        frame_input: <Self::Frame as FrameTr>::FrameInit,
    ) -> Result<
        ItemOrResult<&mut Self::Frame, <Self::Frame as FrameTr>::FrameResult>,
        ContextError<<<Self::Context as ContextTr>::Db as Database>::Error>,
    > {
        self.0.frame_init(frame_input)
    }

    fn frame_run(
        &mut self,
    ) -> Result<
        FrameInitOrResult<Self::Frame>,
        ContextError<<<Self::Context as ContextTr>::Db as Database>::Error>,
    > {
        self.0.frame_run()
    }

    fn frame_return_result(
        &mut self,
        result: <Self::Frame as FrameTr>::FrameResult,
    ) -> Result<
        Option<<Self::Frame as FrameTr>::FrameResult>,
        ContextError<<<Self::Context as ContextTr>::Db as Database>::Error>,
    > {
        self.0.frame_return_result(result)
    }
}
