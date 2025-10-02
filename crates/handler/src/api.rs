use crate::{
    frame::EthFrame, instructions::InstructionProvider, Handler, MainnetHandler, PrecompileProvider,
};
use context::{
    result::{
        EVMError, ExecResultAndState, ExecutionResult, HaltReason, InvalidTransaction,
        ResultAndState, ResultVecAndState, TransactionIndexedError,
    },
    Block, ContextSetters, ContextTr, Database, Evm, JournalTr, Transaction,
};
use database_interface::DatabaseCommit;
use interpreter::{interpreter::EthInterpreter, InterpreterResult};
use state::EvmState;

/// Type alias for the result of transact_many_finalize to reduce type complexity.
type TransactManyFinalizeResult<ExecutionResult, State, Error> =
    Result<ResultVecAndState<ExecutionResult, State>, TransactionIndexedError<Error>>;

/// Execute EVM transactions. Main trait for transaction execution.
pub trait ExecuteEvm {
    /// Output of transaction execution.
    type ExecutionResult;
    /// Output state type representing changes after execution.
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
    fn transact_one(&mut self, tx: Self::Tx) -> Result<Self::ExecutionResult, Self::Error>;

    /// Finalize execution, clearing the journal and returning the accumulated state changes.
    ///
    /// # State Management
    /// Journal is cleared and can be used for next transaction.
    fn finalize(&mut self) -> Self::State;

    /// Transact the given transaction and finalize in a single operation.
    ///
    /// Internally calls [`ExecuteEvm::transact_one`] followed by [`ExecuteEvm::finalize`].
    ///
    /// # Outcome of Error
    ///
    /// If the transaction fails, the journal is considered broken.
    #[inline]
    fn transact(
        &mut self,
        tx: Self::Tx,
    ) -> Result<ExecResultAndState<Self::ExecutionResult, Self::State>, Self::Error> {
        let output_or_error = self.transact_one(tx);
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
    /// If any transaction fails, the journal is finalized and the error is returned with the
    /// transaction index that failed.
    #[inline]
    fn transact_many(
        &mut self,
        txs: impl Iterator<Item = Self::Tx>,
    ) -> Result<std::vec::Vec<Self::ExecutionResult>, TransactionIndexedError<Self::Error>> {
        let mut outputs = std::vec::Vec::new();
        for (index, tx) in txs.enumerate() {
            outputs.push(
                self.transact_one(tx)
                    .inspect_err(|_| {
                        let _ = self.finalize();
                    })
                    .map_err(|error| TransactionIndexedError::new(error, index))?,
            );
        }
        Ok(outputs)
    }

    /// Execute multiple transactions and finalize the state in a single operation.
    ///
    /// Internally calls [`ExecuteEvm::transact_many`] followed by [`ExecuteEvm::finalize`].
    #[inline]
    fn transact_many_finalize(
        &mut self,
        txs: impl Iterator<Item = Self::Tx>,
    ) -> TransactManyFinalizeResult<Self::ExecutionResult, Self::State, Self::Error> {
        // on error transact_multi will clear the journal
        let result = self.transact_many(txs)?;
        let state = self.finalize();
        Ok(ExecResultAndState::new(result, state))
    }

    /// Execute previous transaction and finalize it.
    fn replay(
        &mut self,
    ) -> Result<ExecResultAndState<Self::ExecutionResult, Self::State>, Self::Error>;
}

/// Extension of the [`ExecuteEvm`] trait that adds a method that commits the state after execution.
pub trait ExecuteCommitEvm: ExecuteEvm {
    /// Commit the state.
    fn commit(&mut self, state: Self::State);

    /// Finalize the state and commit it to the database.
    ///
    /// Internally calls `finalize` and `commit` functions.
    #[inline]
    fn commit_inner(&mut self) {
        let state = self.finalize();
        self.commit(state);
    }

    /// Transact the transaction and commit to the state.
    #[inline]
    fn transact_commit(&mut self, tx: Self::Tx) -> Result<Self::ExecutionResult, Self::Error> {
        let output = self.transact_one(tx)?;
        self.commit_inner();
        Ok(output)
    }

    /// Transact multiple transactions and commit to the state.
    ///
    /// Internally calls `transact_many` and `commit_inner` functions.
    #[inline]
    fn transact_many_commit(
        &mut self,
        txs: impl Iterator<Item = Self::Tx>,
    ) -> Result<std::vec::Vec<Self::ExecutionResult>, TransactionIndexedError<Self::Error>> {
        let outputs = self.transact_many(txs)?;
        self.commit_inner();
        Ok(outputs)
    }

    /// Replay the transaction and commit to the state.
    ///
    /// Internally calls `replay` and `commit` functions.
    #[inline]
    fn replay_commit(&mut self) -> Result<Self::ExecutionResult, Self::Error> {
        let result = self.replay()?;
        self.commit(result.state);
        Ok(result.result)
    }
}

impl<CTX, INSP, INST, PRECOMPILES> ExecuteEvm
    for Evm<CTX, INSP, INST, PRECOMPILES, EthFrame<EthInterpreter>>
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

    #[inline]
    fn transact_one(&mut self, tx: Self::Tx) -> Result<Self::ExecutionResult, Self::Error> {
        self.ctx.set_tx(tx);
        MainnetHandler::default().run(self)
    }

    #[inline]
    fn finalize(&mut self) -> Self::State {
        self.journal_mut().finalize()
    }

    #[inline]
    fn set_block(&mut self, block: Self::Block) {
        self.ctx.set_block(block);
    }

    #[inline]
    fn replay(&mut self) -> Result<ResultAndState<HaltReason>, Self::Error> {
        MainnetHandler::default().run(self).map(|result| {
            let state = self.finalize();
            ResultAndState::new(result, state)
        })
    }
}

impl<CTX, INSP, INST, PRECOMPILES> ExecuteCommitEvm
    for Evm<CTX, INSP, INST, PRECOMPILES, EthFrame<EthInterpreter>>
where
    CTX: ContextTr<Journal: JournalTr<State = EvmState>, Db: DatabaseCommit> + ContextSetters,
    INST: InstructionProvider<Context = CTX, InterpreterTypes = EthInterpreter>,
    PRECOMPILES: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    #[inline]
    fn commit(&mut self, state: Self::State) {
        self.db_mut().commit(state);
    }
}
