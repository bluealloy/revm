use crate::{instructions::EthInstructions, EthFrame, Handler, MainnetHandler, PrecompileProvider};
use context::{
    result::{EVMError, ExecutionResult, HaltReason, InvalidTransaction, ResultAndState},
    setters::ContextSetters,
    ContextTr, Database, Evm, Journal,
};
use database_interface::DatabaseCommit;
use interpreter::{interpreter::EthInterpreter, Host, InterpreterResult};
use primitives::Log;
use state::EvmState;
use std::vec::Vec;

/// Execute EVM transactions.
pub trait ExecuteEvm: ContextSetters {
    type Output;

    fn transact_previous(&mut self) -> Self::Output;

    fn transact(&mut self, tx: Self::Tx) -> Self::Output {
        self.set_tx(tx);
        self.transact_previous()
    }
}

/// Execute EVM transactions and commit to the state.
/// TODO this trait can be implemented for all ExecuteEvm for specific Output/CommitOutput
pub trait ExecuteCommitEvm: ExecuteEvm {
    type CommitOutput;

    fn transact_commit_previous(&mut self) -> Self::CommitOutput;

    fn transact_commit(&mut self, tx: Self::Tx) -> Self::CommitOutput {
        self.set_tx(tx);
        self.transact_commit_previous()
    }
}

impl<CTX, INSP, PRECOMPILES> ExecuteEvm
    for Evm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, PRECOMPILES>
where
    CTX: ContextSetters + ContextTr<Journal: Journal<FinalOutput = (EvmState, Vec<Log>)>> + Host,
    PRECOMPILES: PrecompileProvider<Context = CTX, Output = InterpreterResult>,
{
    type Output = Result<
        ResultAndState<HaltReason>,
        EVMError<<CTX::Db as Database>::Error, InvalidTransaction>,
    >;

    fn transact_previous(&mut self) -> Self::Output {
        let mut t = MainnetHandler::<_, _, EthFrame<_, _, _>>::default();
        t.run(self)
    }
}

impl<CTX, INSP, PRECOMPILES> ExecuteCommitEvm
    for Evm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, PRECOMPILES>
where
    CTX: ContextSetters
        + ContextTr<Journal: Journal<FinalOutput = (EvmState, Vec<Log>)>, Db: DatabaseCommit>
        + Host,
    PRECOMPILES: PrecompileProvider<Context = CTX, Output = InterpreterResult>,
{
    type CommitOutput = Result<
        ExecutionResult<HaltReason>,
        EVMError<<CTX::Db as Database>::Error, InvalidTransaction>,
    >;

    fn transact_commit_previous(&mut self) -> Self::CommitOutput {
        self.transact_previous().map(|r| {
            self.db().commit(r.state);
            r.result
        })
    }
}
