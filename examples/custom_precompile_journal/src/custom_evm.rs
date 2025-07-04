//! Custom EVM implementation with journal-accessing precompiles.

use crate::precompile_provider::CustomPrecompileProvider;
use revm::{
    context::{ContextError, ContextSetters, ContextTr, Evm, FrameStack},
    handler::{
        evm::FrameTr, instructions::EthInstructions, EthFrame, EvmTr, FrameInitOrResult,
        ItemOrResult,
    },
    inspector::{InspectorEvmTr, JournalExt},
    interpreter::interpreter::EthInterpreter,
    primitives::hardfork::SpecId,
    Database, Inspector,
};

/// Custom EVM variant with journal-accessing precompiles.
///
/// This EVM extends the standard behavior by using a custom precompile provider
/// that includes journal access functionality. It follows the same pattern as MyEvm
/// but uses CustomPrecompileProvider instead of EthPrecompiles.
#[derive(Debug)]
pub struct CustomEvm<CTX, INSP>(
    pub  Evm<
        CTX,
        INSP,
        EthInstructions<EthInterpreter, CTX>,
        CustomPrecompileProvider,
        EthFrame<EthInterpreter>,
    >,
);

impl<CTX, INSP> CustomEvm<CTX, INSP>
where
    CTX: ContextTr<Cfg: revm::context::Cfg<Spec = SpecId>>,
{
    /// Creates a new instance of CustomEvm with the provided context and inspector.
    ///
    /// # Arguments
    ///
    /// * `ctx` - The execution context that manages state, environment, and journaling
    /// * `inspector` - The inspector for debugging and tracing execution
    ///
    /// # Returns
    ///
    /// A new CustomEvm instance configured with:
    /// - The provided context and inspector
    /// - Mainnet instruction set
    /// - Custom precompiles with journal access
    /// - A fresh frame stack for execution
    pub fn new(ctx: CTX, inspector: INSP) -> Self {
        Self(Evm {
            ctx,
            inspector,
            instruction: EthInstructions::new_mainnet(),
            precompiles: CustomPrecompileProvider::new_with_spec(SpecId::CANCUN),
            frame_stack: FrameStack::new(),
        })
    }
}

impl<CTX, INSP> EvmTr for CustomEvm<CTX, INSP>
where
    CTX: ContextTr<Cfg: revm::context::Cfg<Spec = SpecId>>,
{
    type Context = CTX;
    type Instructions = EthInstructions<EthInterpreter, CTX>;
    type Precompiles = CustomPrecompileProvider;
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

impl<CTX, INSP> InspectorEvmTr for CustomEvm<CTX, INSP>
where
    CTX: ContextSetters<Cfg: revm::context::Cfg<Spec = SpecId>, Journal: JournalExt>,
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
