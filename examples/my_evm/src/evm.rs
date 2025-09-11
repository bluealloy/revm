use revm::{
    context::{ContextError, ContextSetters, ContextTr, Evm, FrameStack},
    handler::{
        evm::FrameTr, instructions::EthInstructions, EthFrame, EthPrecompiles, EvmTr,
        FrameInitOrResult, ItemOrResult,
    },
    inspector::{InspectorEvmTr, JournalExt},
    interpreter::interpreter::EthInterpreter,
    Database, Inspector,
};

/// MyEvm variant of the EVM.
///
/// This struct demonstrates how to create a custom EVM implementation by wrapping
/// the standard REVM components. It combines a context (CTX), an inspector (INSP),
/// and the standard Ethereum instructions, precompiles, and frame execution logic.
///
/// The generic parameters allow for flexibility in the underlying database and
/// inspection capabilities while maintaining the standard Ethereum execution semantics.
#[derive(Debug)]
pub struct MyEvm<CTX, INSP>(
    pub  Evm<
        CTX,
        INSP,
        EthInstructions<EthInterpreter, CTX>,
        EthPrecompiles,
        EthFrame<EthInterpreter>,
    >,
);

impl<CTX: ContextTr, INSP> MyEvm<CTX, INSP> {
    /// Creates a new instance of MyEvm with the provided context and inspector.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The execution context that manages state, environment, and journaling
    /// * `inspector` - The inspector for debugging and tracing execution
    ///
    /// # Returns
    ///
    /// A new MyEvm instance configured with:
    /// - The provided context and inspector
    /// - Mainnet instruction set
    /// - Default Ethereum precompiles
    /// - A fresh frame stack for execution
    pub fn new(ctx: CTX, inspector: INSP) -> Self {
        Self(Evm {
            ctx,
            inspector,
            instruction: EthInstructions::new_mainnet(),
            precompiles: EthPrecompiles::default(),
            frame_stack: FrameStack::new(),
        })
    }
}

impl<CTX: ContextTr, INSP> EvmTr for MyEvm<CTX, INSP>
where
    CTX: ContextTr,
{
    type Context = CTX;
    type Instructions = EthInstructions<EthInterpreter, CTX>;
    type Precompiles = EthPrecompiles;
    type Frame = EthFrame<EthInterpreter>;
    fn ctx(&mut self) -> &mut Self::Context {
        &mut self.0.ctx
    }

    fn ctx_ref(&self) -> &Self::Context {
        self.0.ctx_ref()
    }

    fn ctx_instructions(&mut self) -> (&mut Self::Context, &mut Self::Instructions) {
        self.0.ctx_instructions()
    }

    fn ctx_precompiles(&mut self) -> (&mut Self::Context, &mut Self::Precompiles) {
        self.0.ctx_precompiles()
    }

    fn frame_stack(&mut self) -> &mut FrameStack<Self::Frame> {
        self.0.frame_stack()
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
        frame_result: <Self::Frame as FrameTr>::FrameResult,
    ) -> Result<
        Option<<Self::Frame as FrameTr>::FrameResult>,
        ContextError<<<Self::Context as ContextTr>::Db as Database>::Error>,
    > {
        self.0.frame_return_result(frame_result)
    }
}

impl<CTX, INSP> InspectorEvmTr for MyEvm<CTX, INSP>
where
    CTX: ContextSetters,
    <CTX as ContextSetters>::Journal: JournalExt,
    INSP: Inspector<CTX, EthInterpreter>,
{
    type Inspector = INSP;

    fn inspector(&mut self) -> &mut Self::Inspector {
        self.0.inspector()
    }

    fn ctx_inspector(&mut self) -> (&mut Self::Context, &mut Self::Inspector) {
        self.0.ctx_inspector()
    }

    fn ctx_inspector_frame(
        &mut self,
    ) -> (&mut Self::Context, &mut Self::Inspector, &mut Self::Frame) {
        self.0.ctx_inspector_frame()
    }

    fn ctx_inspector_frame_instructions(
        &mut self,
    ) -> (
        &mut Self::Context,
        &mut Self::Inspector,
        &mut Self::Frame,
        &mut Self::Instructions,
    ) {
        self.0.ctx_inspector_frame_instructions()
    }
}
