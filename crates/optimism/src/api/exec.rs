use crate::{
    evm::OpEvm, handler::OpHandler, transaction::OpTxTrait, L1BlockInfo, OpHaltReason, OpSpec,
    OpTransactionError,
};
use precompile::Log;
use revm::{
    context_interface::{
        result::{EVMError, ExecutionResult, ResultAndState},
        Block, Cfg, ContextTrait, Database, Journal,
    },
    handler::{
        handler::EvmTrait, inspector::Inspector, instructions::EthInstructions, EthFrame,
        EthHandler,
    },
    interpreter::interpreter::EthInterpreter,
    state::EvmState,
    Context, DatabaseCommit, ExecuteCommitEvm, ExecuteEvm, InspectCommitEvm, InspectEvm,
};
use std::vec::Vec;

impl<BLOCK, TX, CFG, DB, JOURNAL, INSP> ExecuteEvm
    for OpEvm<
        Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>,
        INSP,
        EthInstructions<EthInterpreter, Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>>,
    >
where
    BLOCK: Block,
    TX: OpTxTrait,
    CFG: Cfg<Spec = OpSpec>,
    DB: Database,
    JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)>,
    INSP: Inspector<Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>, EthInterpreter>,
{
    type Output =
        Result<ResultAndState<OpHaltReason>, EVMError<<DB as Database>::Error, OpTransactionError>>;

    fn transact_previous(&mut self) -> Self::Output {
        let mut h = OpHandler::<_, _, EthFrame<_, _, _>>::new();
        h.run(self)
    }
}

impl<BLOCK, TX, CFG, DB, JOURNAL, INSP> ExecuteCommitEvm
    for OpEvm<
        Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>,
        INSP,
        EthInstructions<EthInterpreter, Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>>,
    >
where
    BLOCK: Block,
    TX: OpTxTrait,
    CFG: Cfg<Spec = OpSpec>,
    DB: Database + DatabaseCommit,
    JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)>,
    INSP: Inspector<Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>, EthInterpreter>,
{
    type CommitOutput = Result<
        ExecutionResult<OpHaltReason>,
        EVMError<<DB as Database>::Error, OpTransactionError>,
    >;

    fn transact_commit_previous(&mut self) -> Self::CommitOutput {
        self.transact_previous().map(|r| {
            self.ctx().db().commit(r.state);
            r.result
        })
    }
}

impl<BLOCK, TX, CFG, DB, JOURNAL, INSP> InspectEvm
    for OpEvm<
        Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>,
        INSP,
        EthInstructions<EthInterpreter, Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>>,
    >
where
    BLOCK: Block,
    TX: OpTxTrait,
    CFG: Cfg<Spec = OpSpec>,
    DB: Database,
    JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)>,
    INSP: Inspector<Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>, EthInterpreter>,
{
    type Inspector = INSP;

    fn set_inspector(&mut self, inspector: Self::Inspector) {
        self.data.inspector = inspector;
    }

    fn inspect_previous(&mut self) -> Self::Output {
        let mut h = OpHandler::<_, _, EthFrame<_, _, _>>::new();
        h.run(self)
    }
}

impl<BLOCK, TX, CFG, DB, JOURNAL, INSP> InspectCommitEvm
    for OpEvm<
        Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>,
        INSP,
        EthInstructions<EthInterpreter, Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>>,
    >
where
    BLOCK: Block,
    TX: OpTxTrait,
    CFG: Cfg<Spec = OpSpec>,
    DB: Database + DatabaseCommit,
    JOURNAL: Journal<Database = DB, FinalOutput = (EvmState, Vec<Log>)>,
    INSP: Inspector<Context<BLOCK, TX, CFG, DB, JOURNAL, L1BlockInfo>, EthInterpreter>,
{
    fn inspect_commit_previous(&mut self) -> Self::CommitOutput {
        self.inspect_previous().map(|r| {
            self.ctx().db().commit(r.state);
            r.result
        })
    }
}
