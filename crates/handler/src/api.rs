use crate::{
    instructions::InstructionProvider, EthFrame, Handler, MainnetHandler, PrecompileProvider,
};
use context::{
    result::{EVMError, ExecutionResult, HaltReason, InvalidTransaction},
    Block, ContextSetters, ContextTr, Database, Evm, JournalTr, Transaction,
};
use database_interface::DatabaseCommit;
use interpreter::{interpreter::EthInterpreter, InterpreterResult};
use state::EvmState;

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
    /// # State Management
    /// State changes are stored in the internal journal.
    /// To retrieve the state, call [`ExecuteEvm::finalize`] after transaction execution.
    ///
    /// # History Note
    /// Previously this function returned both output and state.
    /// Now it follows a two-step process: execute then finalize.
    fn transact(&mut self, tx: Self::Tx) -> Result<Self::ExecutionResult, Self::Error>;

    /// Finalize execution, clearing the journal and returning the accumulated state changes.
    fn finalize(&mut self) -> Self::State;

    /// Transact the given transaction and finalize in a single operation.
    ///
    /// Internally calls [`ExecuteEvm::transact`] followed by [`ExecuteEvm::finalize`].
    fn transact_finalize(
        &mut self,
        tx: Self::Tx,
    ) -> Result<(Self::ExecutionResult, Self::State), Self::Error> {
        let output = self.transact(tx)?;
        let state = self.finalize();
        Ok((output, state))
    }

    /// Execute multiple transactions without finalizing the state.
    ///
    /// Returns a vector of execution results. State changes are accumulated in the journal
    /// but not finalized. Call [`ExecuteEvm::finalize`] after execution to retrieve state changes.
    fn transact_multi(
        &mut self,
        txs: impl Iterator<Item = Self::Tx>,
    ) -> Result<Vec<Self::ExecutionResult>, Self::Error> {
        let mut outputs = Vec::new();
        for tx in txs {
            outputs.push(self.transact(tx)?);
        }
        Ok(outputs)
    }

    /// Execute multiple transactions and finalize the state in a single operation.
    ///
    /// Internally calls [`ExecuteEvm::transact_multi`] followed by [`ExecuteEvm::finalize`].
    fn transact_multi_finalize(
        &mut self,
        txs: impl Iterator<Item = Self::Tx>,
    ) -> Result<(Vec<Self::ExecutionResult>, Self::State), Self::Error> {
        let output = self.transact_multi(txs)?;
        let state = self.finalize();
        Ok((output, state))
    }

    /// Reverts the most recent transaction in the journal.
    ///
    /// Pops the last transaction from the journal, reverting all state changes made by that transaction.
    /// If the journal is empty, this method does nothing.
    fn revert(&mut self);

    /// Reverts all transactions in the journal, clearing it completely.
    fn revert_all(&mut self);
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
        let (output, state) = self.transact_finalize(tx)?;
        self.commit(state);
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
        let state = self.finalize();
        self.commit(state);
        Ok(outputs)
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

    fn revert(&mut self) {
        self.journal().revert_tx();
    }

    fn revert_all(&mut self) {
        todo!();
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
