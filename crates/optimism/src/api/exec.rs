use precompile::Log;
use revm::{
    context_interface::{
        result::{EVMError, ExecutionResult, ResultAndState},
        Block, Cfg, ContextGetters, Database, Journal,
    },
    handler::{handler::EvmTypesTrait, inspector::Inspector, instructions::EthInstructions},
    interpreter::interpreter::EthInterpreter,
    state::EvmState,
    Context, DatabaseCommit, ExecuteCommitEvm, ExecuteEvm,
};

use crate::{
    evm::{transact_op, OpEvm},
    transaction::OpTxTrait,
    L1BlockInfo, OpHaltReason, OpSpec, OpTransactionError,
};

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

    fn exec_previous(&mut self) -> Self::Output {
        transact_op(self)
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

    fn exec_commit_previous(&mut self) -> Self::CommitOutput {
        transact_op(self).map(|r| {
            self.ctx().db().commit(r.state);
            r.result
        })
    }
}
