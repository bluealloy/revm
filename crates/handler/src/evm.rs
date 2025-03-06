use crate::{instructions::EthInstructions, EthFrame, Handler, MainnetHandler, PrecompileProvider};
use context::{
    result::{EVMError, ExecutionResult, HaltReason, InvalidTransaction, ResultAndState},
    setters::ContextSetters,
    ContextTr, Database, Evm, Journal,
};
use database_interface::DatabaseCommit;
use interpreter::{interpreter::EthInterpreter, InterpreterResult};
use primitives::Log;
use state::EvmState;
use std::vec::Vec;

/// Execute EVM transactions.
pub trait ExecuteEvm: ContextSetters {
    /// Output of transaction execution.
    type Output;

    fn transact_previous(&mut self) -> Self::Output;

    fn transact(&mut self, tx: Self::Tx) -> Self::Output {
        self.set_tx(tx);
        self.transact_previous()
    }
}

/// Execute EVM transactions and commit to the state.
pub trait ExecuteCommitEvm: ExecuteEvm {
    /// Commit output of transaction execution.
    type CommitOutput;

    /// Transact the transaction and commit to the state.
    fn transact_commit_previous(&mut self) -> Self::CommitOutput;

    /// Transact the transaction and commit to the state.
    fn transact_commit(&mut self, tx: Self::Tx) -> Self::CommitOutput {
        self.set_tx(tx);
        self.transact_commit_previous()
    }
}

impl<CTX, INSP, PRECOMPILES> ExecuteEvm
    for Evm<CTX, INSP, EthInstructions<EthInterpreter, CTX>, PRECOMPILES>
where
    CTX: ContextSetters + ContextTr<Journal: Journal<FinalOutput = (EvmState, Vec<Log>)>>,
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
        + ContextTr<Journal: Journal<FinalOutput = (EvmState, Vec<Log>)>, Db: DatabaseCommit>,
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
