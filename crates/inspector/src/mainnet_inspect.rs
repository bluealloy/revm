use crate::{
    handler::{frame_end, frame_start},
    inspect::{InspectCommitEvm, InspectEvm},
    inspect_instructions,
    Inspector,
    InspectorEvmTr,
    InspectorFrame,
    InspectorHandler,
    JournalExt,
};
use context::{ContextSetters, ContextTr, Evm, JournalTr};
use database_interface::DatabaseCommit;
use handler::{
    evm::{ContextDbError, FrameInitResult},
    instructions::InstructionProvider,
    EthFrame,
    EvmTr,
    EvmTrError,
    FrameInitOrResult,
    FrameTr,
    Handler,
    ItemOrResult,
    MainnetHandler,
    PrecompileProvider,
};
use interpreter::{interpreter::EthInterpreter, InterpreterResult};
use state::EvmState;

// Implementing InspectorHandler for MainnetHandler.
impl<EVM, ERROR> InspectorHandler for MainnetHandler<EVM, ERROR, EthFrame<EthInterpreter>>
where
    EVM: InspectorEvmTr<
        Context: ContextTr<Journal: JournalTr<State = EvmState>>,
        Frame = EthFrame<EthInterpreter>,
        Inspector: Inspector<<<Self as Handler>::Evm as EvmTr>::Context, EthInterpreter>,
    >,
    ERROR: EvmTrError<EVM>,
{
    type IT = EthInterpreter;
}

// Implementing InspectEvm for Evm
impl<CTX, INSP, INST, PRECOMPILES> InspectEvm
    for Evm<CTX, INSP, INST, PRECOMPILES, EthFrame<EthInterpreter>>
where
    CTX: ContextSetters + ContextTr<Journal: JournalTr<State = EvmState> + JournalExt>,
    INSP: Inspector<CTX, EthInterpreter>,
    INST: InstructionProvider<Context = CTX, InterpreterTypes = EthInterpreter>,
    PRECOMPILES: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    type Inspector = INSP;

    fn set_inspector(&mut self, inspector: Self::Inspector) {
        self.inspector = inspector;
    }

    fn inspect_one_tx(&mut self, tx: Self::Tx) -> Result<Self::ExecutionResult, Self::Error> {
        self.set_tx(tx);
        MainnetHandler::default().inspect_run(self)
    }
}

// Implementing InspectCommitEvm for Evm
impl<CTX, INSP, INST, PRECOMPILES> InspectCommitEvm
    for Evm<CTX, INSP, INST, PRECOMPILES, EthFrame<EthInterpreter>>
where
    CTX: ContextSetters
        + ContextTr<Journal: JournalTr<State = EvmState> + JournalExt, Db: DatabaseCommit>,
    INSP: Inspector<CTX, EthInterpreter>,
    INST: InstructionProvider<Context = CTX, InterpreterTypes = EthInterpreter>,
    PRECOMPILES: PrecompileProvider<CTX, Output = InterpreterResult>,
{
}

// Implementing InspectorEvmTr for Evm
impl<CTX, INSP, I, P> InspectorEvmTr for Evm<CTX, INSP, I, P, EthFrame<EthInterpreter>>
where
    CTX: ContextTr<Journal: JournalExt> + ContextSetters,
    I: InstructionProvider<Context = CTX, InterpreterTypes = EthInterpreter>,
    P: PrecompileProvider<CTX, Output = InterpreterResult>,
    INSP: Inspector<CTX, I::InterpreterTypes>,
{
    type Inspector = INSP;

    fn inspector(&mut self) -> &mut Self::Inspector {
        &mut self.inspector
    }

    fn ctx_inspector(&mut self) -> (&mut Self::Context, &mut Self::Inspector) {
        (&mut self.ctx, &mut self.inspector)
    }

    fn ctx_inspector_frame(
        &mut self,
    ) -> (&mut Self::Context, &mut Self::Inspector, &mut Self::Frame) {
        (&mut self.ctx, &mut self.inspector, self.frame_stack.get())
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
            &mut self.ctx,
            &mut self.inspector,
            self.frame_stack.get(),
            &mut self.instruction,
        )
    }

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
        let interp = frame.interpreter();
        inspector.initialize_interp(interp, ctx);
        Ok(ItemOrResult::Item(frame))
    }

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
