use crate::{
    handler::inspect_instructions,
    inspect::{InspectCommitEvm, InspectEvm},
    Inspector, InspectorEvmTr, InspectorFrame, InspectorHandler, JournalExt,
};
use context::{ContextSetters, ContextTr, Evm, JournalOutput, JournalTr};
use database_interface::DatabaseCommit;
use handler::{
    instructions::InstructionProvider, EthFrame, EvmTr, EvmTrError, Frame, FrameResult, Handler,
    MainnetHandler, PrecompileProvider,
};
use interpreter::{
    interpreter::EthInterpreter, FrameInput, Interpreter, InterpreterAction, InterpreterResult,
    InterpreterTypes,
};

// Implementing InspectorHandler for MainnetHandler.
impl<EVM, ERROR, FRAME> InspectorHandler for MainnetHandler<EVM, ERROR, FRAME>
where
    EVM: InspectorEvmTr<
        Context: ContextTr<Journal: JournalTr<FinalOutput = JournalOutput>>,
        Inspector: Inspector<<<Self as Handler>::Evm as EvmTr>::Context, EthInterpreter>,
    >,
    ERROR: EvmTrError<EVM>,
    FRAME: Frame<Evm = EVM, Error = ERROR, FrameResult = FrameResult, FrameInit = FrameInput>
        + InspectorFrame<IT = EthInterpreter>,
{
    type IT = EthInterpreter;
}

// Implementing InspectEvm for Evm
impl<CTX, INSP, INST, PRECOMPILES> InspectEvm for Evm<CTX, INSP, INST, PRECOMPILES>
where
    CTX: ContextSetters + ContextTr<Journal: JournalTr<FinalOutput = JournalOutput> + JournalExt>,
    INSP: Inspector<CTX, EthInterpreter>,
    INST: InstructionProvider<Context = CTX, InterpreterTypes = EthInterpreter>,
    PRECOMPILES: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    type Inspector = INSP;

    fn set_inspector(&mut self, inspector: Self::Inspector) {
        self.inspector = inspector;
    }

    fn inspect_replay(&mut self) -> Self::Output {
        let mut t = MainnetHandler::<_, _, EthFrame<_, _, _>> {
            _phantom: core::marker::PhantomData,
        };

        t.inspect_run(self)
    }
}

// Implementing InspectCommitEvm for Evm
impl<CTX, INSP, INST, PRECOMPILES> InspectCommitEvm for Evm<CTX, INSP, INST, PRECOMPILES>
where
    CTX: ContextSetters
        + ContextTr<Journal: JournalTr<FinalOutput = JournalOutput> + JournalExt, Db: DatabaseCommit>,
    INSP: Inspector<CTX, EthInterpreter>,
    INST: InstructionProvider<Context = CTX, InterpreterTypes = EthInterpreter>,
    PRECOMPILES: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    fn inspect_replay_commit(&mut self) -> Self::CommitOutput {
        self.inspect_replay().map(|r| {
            self.ctx().db().commit(r.state);
            r.result
        })
    }
}

// Implementing InspectorEvmTr for Evm
impl<CTX, INSP, I, P> InspectorEvmTr for Evm<CTX, INSP, I, P>
where
    CTX: ContextTr<Journal: JournalExt> + ContextSetters,
    I: InstructionProvider<
        Context = CTX,
        InterpreterTypes: InterpreterTypes<Output = InterpreterAction>,
    >,
    P: PrecompileProvider<CTX>,
    INSP: Inspector<CTX, I::InterpreterTypes>,
{
    type Inspector = INSP;

    fn inspector(&mut self) -> &mut Self::Inspector {
        &mut self.inspector
    }

    fn ctx_inspector(&mut self) -> (&mut Self::Context, &mut Self::Inspector) {
        (&mut self.ctx, &mut self.inspector)
    }

    fn run_inspect_interpreter(
        &mut self,
        interpreter: &mut Interpreter<
            <Self::Instructions as InstructionProvider>::InterpreterTypes,
        >,
    ) -> <<Self::Instructions as InstructionProvider>::InterpreterTypes as InterpreterTypes>::Output
    {
        let context = &mut self.ctx;
        let instructions = &mut self.instruction;
        let inspector = &mut self.inspector;

        inspect_instructions(
            context,
            interpreter,
            inspector,
            instructions.instruction_table(),
        )
    }
}
