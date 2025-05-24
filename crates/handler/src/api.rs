use crate::{
    instructions::InstructionProvider, EthFrame, Handler, MainnetHandler, PrecompileProvider,
};
use context::{
    result::{
        EVMError, ExecutionResult, HaltReason, InvalidTransaction, ResultAndState,
        ResultVecAndState,
    },
    Block, ContextSetters, ContextTr, Database, Evm, JournalTr, Transaction,
};
use database_interface::DatabaseCommit;
use interpreter::{interpreter::EthInterpreter, InterpreterResult};
use state::EvmState;
use std::vec::Vec;

/// Execute EVM transactions. Main trait for transaction execution.
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
    /// For quicker error handling, use [`ExecuteEvm::transact_finalize`] that will drop the journal.
    ///
    /// # State Management
    /// State changes are stored in the internal journal.
    /// To retrieve the state, call [`ExecuteEvm::finalize`] after transaction execution.
    ///
    /// # History Note
    /// Previously this function returned both output and state.
    /// Now it follows a two-step process: execute then finalize.
    fn transact(&mut self, tx: Self::Tx) -> Result<Self::ExecutionResult, Self::Error>;

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
    fn transact_finalize(
        &mut self,
        tx: Self::Tx,
    ) -> Result<ResultAndState<Self::ExecutionResult, Self::State>, Self::Error> {
        let output_or_error = self.transact(tx);
        // finalize will clear the journal
        let state = self.finalize();
        let output = output_or_error?;
        Ok(ResultAndState::new(output, state))
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
    fn transact_multi(
        &mut self,
        txs: impl Iterator<Item = Self::Tx>,
    ) -> Result<Vec<Self::ExecutionResult>, Self::Error> {
        let mut outputs = Vec::new();
        for tx in txs {
            outputs.push(self.transact(tx).inspect_err(|_| {
                let _ = self.finalize();
            })?);
        }
        Ok(outputs)
    }

    /// Execute multiple transactions and finalize the state in a single operation.
    ///
    /// Internally calls [`ExecuteEvm::transact_multi`] followed by [`ExecuteEvm::finalize`].
    //#[allow(clippy::type_complexity)]
    fn transact_multi_finalize(
        &mut self,
        txs: impl Iterator<Item = Self::Tx>,
    ) -> Result<ResultVecAndState<Self::ExecutionResult, Self::State>, Self::Error> {
        // on error transact_multi will clear the journal
        let result = self.transact_multi(txs)?;
        let state = self.finalize();
        Ok(ResultAndState::new(result, state))
    }

    /// Execute previous transaction and finalize it.
    ///
    /// Doint it without finalization
    fn replay(&mut self)
        -> Result<ResultAndState<Self::ExecutionResult, Self::State>, Self::Error>;
}

/// Extension of the [`ExecuteEvm`] trait that adds a method that commits the state after execution.
pub trait ExecuteCommitEvm: ExecuteEvm {
    /// Commit the state.
    fn commit(&mut self, state: Self::State);

    /// Finalize the state and commit it to the database.
    ///
    /// Internally calls `finalize` and `commit` functions.
    fn commit_inner(&mut self) {
        let state = self.finalize();
        self.commit(state);
    }

    /// Transact the transaction and commit to the state.
    fn transact_commit(&mut self, tx: Self::Tx) -> Result<Self::ExecutionResult, Self::Error> {
        let output = self.transact(tx)?;
        self.commit_inner();
        Ok(output)
    }

    /// Transact multiple transactions and commit to the state.
    ///
    /// Internally calls `transact_multi` and `commit` functions.
    fn transact_multi_commit(
        &mut self,
        txs: impl Iterator<Item = Self::Tx>,
    ) -> Result<Vec<Self::ExecutionResult>, Self::Error> {
        let outputs = self.transact_multi(txs)?;
        self.commit_inner();
        Ok(outputs)
    }

    /// Replay the transaction and commit to the state.
    ///
    /// Internally calls `replay` and `commit` functions.
    fn replay_commit(&mut self) -> Result<Self::ExecutionResult, Self::Error> {
        let result = self.replay()?;
        self.commit(result.state);
        Ok(result.result)
    }
}

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

    fn transact(&mut self, tx: Self::Tx) -> Result<Self::ExecutionResult, Self::Error> {
        self.ctx.set_tx(tx);
        let mut t = MainnetHandler::<_, _, EthFrame<_, _, _>>::default();
        t.run(self)
    }

    fn finalize(&mut self) -> Self::State {
        self.journal().finalize()
    }

    fn set_block(&mut self, block: Self::Block) {
        self.ctx.set_block(block);
    }

    fn replay(
        &mut self,
    ) -> Result<ResultAndState<Self::ExecutionResult, Self::State>, Self::Error> {
        let mut t = MainnetHandler::<_, _, EthFrame<_, _, _>>::default();
        t.run(self).map(|result| {
            let state = self.finalize();
            ResultAndState::new(result, state)
        })
    }
}

impl<CTX, INSP, INST, PRECOMPILES> ExecuteCommitEvm for Evm<CTX, INSP, INST, PRECOMPILES>
where
    CTX: ContextTr<Journal: JournalTr<State = EvmState>, Db: DatabaseCommit> + ContextSetters,
    INST: InstructionProvider<Context = CTX, InterpreterTypes = EthInterpreter>,
    PRECOMPILES: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    fn commit(&mut self, state: Self::State) {
        self.db().commit(state);
    }
}
