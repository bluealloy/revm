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
    /// Previously this function returned both output and state. Now it returns only the output and the state
    /// can be obtained by calling `finalize` function. Function with same behavior is `transact_finalize`.
    fn transact(&mut self, tx: Self::Tx) -> Result<Self::ExecutionResult, Self::Error>;

    /// Finalize execution, clearing journal and returning state.
    fn finalize(&mut self) -> Self::State;

    /// Transact the given transaction.
    ///
    /// Internally calls combo of `transact_continue` and `finalize` functions.
    fn transact_finalize(
        &mut self,
        tx: Self::Tx,
    ) -> Result<(Self::ExecutionResult, Self::State), Self::Error> {
        let output = self.transact(tx)?;
        let state = self.finalize();
        Ok((output, state))
    }

    /// Execute multiple transaction without finalizing.
    ///
    /// This method offers adding additional transactions to the execution, or allow execution last transaction
    /// with Inspect mode.
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

    /// Execute multiple transaction and finalize.
    ///
    /// Finalization returns both the list of execution results and all state changes from execution.
    fn transact_multi_finalize(
        &mut self,
        txs: impl Iterator<Item = Self::Tx>,
    ) -> Result<(Vec<Self::ExecutionResult>, Self::State), Self::Error> {
        let output = self.transact_multi(txs)?;
        let state = self.finalize();
        Ok((output, state))
    }

    /// Pops last transaction from journal, reverting state to previous transaction.
    ///
    /// In case there is no transaction to pop, it does nothing.
    fn revert(&mut self);

    /// Pops all transactions from journal, clearing it.
    fn revert_all(&mut self);
}

/// Extension of the [`ExecuteEvm`] trait that adds a method that commits the state after execution.
pub trait ExecuteCommitEvm: ExecuteEvm {
    fn commit(&mut self, state: Self::State);

    /// Transact the transaction and commit to the state.
    fn transact_commit(&mut self, tx: Self::Tx) -> Result<Self::ExecutionResult, Self::Error> {
        let (output, state) = self.transact_finalize(tx)?;
        self.commit(state);
        Ok(output)
    }

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
        todo!("");
        //self.journal().finalize()
    }

    fn set_block(&mut self, block: Self::Block) {
        self.ctx.set_block(block);
    }

    fn revert(&mut self) {
        todo!();
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
