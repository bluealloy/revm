use crate::handler::Handler;
use crate::{instructions::InstructionProvider, EthFrame, MainnetHandler, PrecompileProvider};
use async_trait::async_trait;
use context::{
    result::{
        EVMError, ExecResultAndState, ExecutionResult, HaltReason, InvalidTransaction,
        ResultVecAndState,
    },
    Block, ContextSetters, ContextTr, Database, Evm, JournalTr, Transaction,
};
use database_interface::DatabaseCommit;
use interpreter::{interpreter::EthInterpreter, InterpreterResult};
use state::EvmState;
use std::vec::Vec;

/// Execute EVM transactions. Main trait for transaction execution.
#[async_trait(?Send)]
pub trait ExecuteEvm {
    /// Output of transaction execution.
    type ExecutionResult;
    // Output state
    type State;
    /// Error type
    type Error;
    /// Transaction type.
    type Tx: Transaction;
    /// Block type.
    type Block: Block;

    /// Set the block.
    fn set_block(&mut self, block: Self::Block);

    /// Execute transaction and store state inside journal. Returns output of transaction execution.
    ///
    /// # Return Value
    /// Returns only the execution result
    ///
    /// # Error Handling
    /// If the transaction fails, the journal will revert all changes of given transaction.
    /// For quicker error handling, use [`ExecuteEvm::transact`] that will drop the journal.
    ///
    /// # State Management
    /// State changes are stored in the internal journal.
    /// To retrieve the state, call [`ExecuteEvm::finalize`] after transaction execution.
    ///
    /// # History Note
    /// Previously this function returned both output and state.
    /// Now it follows a two-step process: execute then finalize.
    async fn transact_one(&mut self, tx: Self::Tx) -> Result<Self::ExecutionResult, Self::Error>;

    /// Finalize execution, clearing the journal and returning the accumulated state changes.
    ///
    /// # State Management
    /// Journal is cleared and can be used for next transaction.
    fn finalize(&mut self) -> Self::State;

    /// Transact the given transaction and finalize in a single operation.
    ///
    /// Internally calls [`ExecuteEvm::transact`] followed by [`ExecuteEvm::finalize`].
    ///
    /// # Outcome of Error
    ///
    /// If the transaction fails, the journal is considered broken.
    async fn transact(
        &mut self,
        tx: Self::Tx,
    ) -> Result<ExecResultAndState<Self::ExecutionResult, Self::State>, Self::Error> {
        let output_or_error = self.transact_one(tx).await;
        // finalize will clear the journal
        let state = self.finalize();
        let output = output_or_error?;
        Ok(ExecResultAndState::new(output, state))
    }

    /// Execute multiple transactions without finalizing the state.
    ///
    /// Returns a vector of execution results. State changes are accumulated in the journal
    /// but not finalized. Call [`ExecuteEvm::finalize`] after execution to retrieve state changes.
    ///
    /// # Outcome of Error
    ///
    /// If any transaction fails, the journal is finalized and the last error is returned.
    ///
    /// TODO add tx index to the error.
    async fn transact_many(
        &mut self,
        txs: impl Iterator<Item = Self::Tx>,
    ) -> Result<Vec<Self::ExecutionResult>, Self::Error> {
        let mut outputs = Vec::new();
        for tx in txs {
            outputs.push(self.transact_one(tx).await.inspect_err(|_| {
                let _ = self.finalize();
            })?);
        }
        Ok(outputs)
    }

    /// Execute multiple transactions and finalize the state in a single operation.
    ///
    /// Internally calls [`ExecuteEvm::transact_many`] followed by [`ExecuteEvm::finalize`].
    //#[allow(clippy::type_complexity)]
    async fn transact_many_finalize(
        &mut self,
        txs: impl Iterator<Item = Self::Tx>,
    ) -> Result<ResultVecAndState<Self::ExecutionResult, Self::State>, Self::Error> {
        // on error transact_multi will clear the journal
        let result = self.transact_many(txs).await?;
        let state = self.finalize();
        Ok(ExecResultAndState::new(result, state))
    }

    /// Execute the previously executed transaction again ("replay") and finalize it.
    ///
    /// Returns the execution result together with the finalized state changes.
    async fn replay(
        &mut self,
    ) -> Result<ExecResultAndState<Self::ExecutionResult, Self::State>, Self::Error>;
}

/// Extension of the [`ExecuteEvm`] trait that adds helper methods which *commit* the
/// finalized state back to the underlying database.  All functions are now async so
/// callers can move to an executor without relying on `poll_to_ready`.
#[async_trait(?Send)]
pub trait ExecuteCommitEvm: ExecuteEvm {
    /// Commit the finalized state to the database.
    async fn commit(&mut self, state: Self::State);

    /// Finalize the state and commit it to the database.
    ///
    /// Internally calls `finalize` then [`Self::commit`].
    async fn commit_inner(&mut self) {
        let state = self.finalize();
        self.commit(state).await;
    }

    /// Execute a single transaction and commit the resulting state.
    async fn transact_commit(
        &mut self,
        tx: Self::Tx,
    ) -> Result<Self::ExecutionResult, Self::Error> {
        let output = self.transact_one(tx).await?;
        self.commit_inner().await;
        Ok(output)
    }

    /// Execute multiple transactions and commit the accumulated state.
    async fn transact_many_commit(
        &mut self,
        txs: impl Iterator<Item = Self::Tx>,
    ) -> Result<Vec<Self::ExecutionResult>, Self::Error> {
        let outputs = self.transact_many(txs).await?;
        self.commit_inner().await;
        Ok(outputs)
    }

    /// Replay the last executed transaction and commit.
    async fn replay_commit(&mut self) -> Result<Self::ExecutionResult, Self::Error> {
        let result = self.replay().await?;
        self.commit(result.state).await;
        Ok(result.result)
    }
}

#[async_trait(?Send)]
impl<CTX, INSP, INST, PRECOMPILES> ExecuteEvm for Evm<CTX, INSP, INST, PRECOMPILES>
where
    CTX: ContextTr<Journal: JournalTr<State = EvmState>> + ContextSetters,
    INST: InstructionProvider<Context = CTX, InterpreterTypes = EthInterpreter>,
    PRECOMPILES: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    type ExecutionResult = ExecutionResult<HaltReason>;
    type State = EvmState;
    type Error = EVMError<<CTX::Db as Database>::Error, InvalidTransaction>;
    type Tx = <CTX as ContextTr>::Tx;
    type Block = <CTX as ContextTr>::Block;

    async fn transact_one(&mut self, tx: Self::Tx) -> Result<Self::ExecutionResult, Self::Error> {
        self.ctx.set_tx(tx);
        let mut t = MainnetHandler::<_, _, EthFrame<_, _, _>>::default();
        t.run(self).await
    }

    fn finalize(&mut self) -> Self::State {
        self.journal_mut().finalize()
    }

    fn set_block(&mut self, block: Self::Block) {
        self.ctx.set_block(block);
    }

    async fn replay(
        &mut self,
    ) -> Result<ExecResultAndState<Self::ExecutionResult, Self::State>, Self::Error> {
        let mut t = MainnetHandler::<_, _, EthFrame<_, _, _>>::default();
        t.run(self).await.map(|result| {
            let state = self.finalize();
            ExecResultAndState::new(result, state)
        })
    }
}

#[async_trait(?Send)]
impl<CTX, INSP, INST, PRECOMPILES> ExecuteCommitEvm for Evm<CTX, INSP, INST, PRECOMPILES>
where
    CTX: ContextTr<Journal: JournalTr<State = EvmState>, Db: DatabaseCommit> + ContextSetters,
    INST: InstructionProvider<Context = CTX, InterpreterTypes = EthInterpreter>,
    PRECOMPILES: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    async fn commit(&mut self, state: Self::State) {
        // The underlying database commit is already async – simply forward and await.
        self.db_mut().commit(state).await;
    }
}
