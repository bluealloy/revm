use crate::{
    evm::OpEvm, handler::OpHandler, transaction::OpTxTr, L1BlockInfo, OpHaltReason, OpSpecId,
    OpTransactionError,
};
use revm::{
    context::JournalOutput,
    context_interface::{
        result::{EVMError, ExecutionResult, ResultAndState},
        Block, Cfg, ContextTr, Database, JournalTr,
    },
    handler::{instructions::EthInstructions, EthFrame, EvmTr, Handler, PrecompileProvider},
    inspector::{InspectCommitEvm, InspectEvm, Inspector, JournalExt},
    interpreter::{interpreter::EthInterpreter, InterpreterResult},
    Context, DatabaseCommit, ExecuteCommitEvm, ExecuteEvm,
};

impl<BLOCK, TX, CFG, DB, JOURNAL, INSP, PRECOMPILE> ExecuteEvm
    for OpEvm<
        Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>,
        INSP,
        EthInstructions<EthInterpreter, Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>>,
        PRECOMPILE,
    >
where
    BLOCK: Block,
    TX: OpTxTr,
    CFG: Cfg<Spec = OpSpecId>,
    DB: Database,
    JOURNAL: JournalTr<Database = DB, FinalOutput = JournalOutput>,
    PRECOMPILE: PrecompileProvider<
        Context = Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>,
        Output = InterpreterResult,
    >,
{
    type Output =
        Result<ResultAndState<OpHaltReason>, EVMError<<DB as Database>::Error, OpTransactionError>>;

    fn replay(&mut self) -> Self::Output {
        let mut h = OpHandler::<_, _, EthFrame<_, _, _>>::new();
        h.run(self)
    }
}

impl<BLOCK, TX, CFG, DB, JOURNAL, INSP, PRECOMPILE> ExecuteCommitEvm
    for OpEvm<
        Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>,
        INSP,
        EthInstructions<EthInterpreter, Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>>,
        PRECOMPILE,
    >
where
    BLOCK: Block,
    TX: OpTxTr,
    CFG: Cfg<Spec = OpSpecId>,
    DB: Database + DatabaseCommit,
    JOURNAL: JournalTr<Database = DB, FinalOutput = JournalOutput> + JournalExt,
    PRECOMPILE: PrecompileProvider<
        Context = Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>,
        Output = InterpreterResult,
    >,
{
    type CommitOutput = Result<
        ExecutionResult<OpHaltReason>,
        EVMError<<DB as Database>::Error, OpTransactionError>,
    >;

    fn replay_commit(&mut self) -> Self::CommitOutput {
        self.replay().map(|r| {
            self.ctx().db().commit(r.state);
            r.result
        })
    }
}

impl<BLOCK, TX, CFG, DB, JOURNAL, INSP, PRECOMPILE> InspectEvm
    for OpEvm<
        Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>,
        INSP,
        EthInstructions<EthInterpreter, Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>>,
        PRECOMPILE,
    >
where
    BLOCK: Block,
    TX: OpTxTr,
    CFG: Cfg<Spec = OpSpecId>,
    DB: Database,
    JOURNAL: JournalTr<Database = DB, FinalOutput = JournalOutput> + JournalExt,
    INSP: Inspector<Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>, EthInterpreter>,
    PRECOMPILE: PrecompileProvider<
        Context = Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>,
        Output = InterpreterResult,
    >,
{
    type Inspector = INSP;

    fn set_inspector(&mut self, inspector: Self::Inspector) {
        self.0.data.inspector = inspector;
    }

    fn inspect_previous(&mut self) -> Self::Output {
        let mut h = OpHandler::<_, _, EthFrame<_, _, _>>::new();
        h.run(self)
    }
}

impl<BLOCK, TX, CFG, DB, JOURNAL, INSP, PRECOMPILE> InspectCommitEvm
    for OpEvm<
        Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>,
        INSP,
        EthInstructions<EthInterpreter, Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>>,
        PRECOMPILE,
    >
where
    BLOCK: Block,
    TX: OpTxTr,
    CFG: Cfg<Spec = OpSpecId>,
    DB: Database + DatabaseCommit,
    JOURNAL: JournalTr<Database = DB, FinalOutput = JournalOutput> + JournalExt,
    INSP: Inspector<Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>, EthInterpreter>,
    PRECOMPILE: PrecompileProvider<
        Context = Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>,
        Output = InterpreterResult,
    >,
{
    fn inspect_commit_previous(&mut self) -> Self::CommitOutput {
        self.inspect_previous().map(|r| {
            self.ctx().db().commit(r.state);
            r.result
        })
    }
}
