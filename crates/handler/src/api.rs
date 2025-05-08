use crate::{
    instructions::InstructionProvider, EthFrame, Handler, MainnetHandler, PrecompileProvider,
};
use context::{
    result::{EVMError, ExecutionResult, HaltReason, InvalidTransaction, ResultAndState},
    Block, ContextSetters, ContextTr, Database, Evm, JournalOutput, JournalTr, Transaction,
};
use database_interface::DatabaseCommit;
use interpreter::{interpreter::EthInterpreter, InterpreterResult};
use state::EvmState;

/// Execute EVM transactions. Main trait for transaction execution.
pub trait ExecuteEvm {
    /// Output of transaction execution.
    type Output;
    // Output state
    type State;
    /// Error type
    type Error;
    /// Transaction type.
    type Tx: Transaction;
    /// Block type.
    type Block: Block;

    /// Set the transaction.
    fn set_tx(&mut self, tx: Self::Tx);

    /// Set the block.
    fn set_block(&mut self, block: Self::Block);

    fn transact_continue(&mut self, tx: Self::Tx) -> Result<Self::Output, Self::Error>;

    fn finalize(&mut self) -> Self::State;

    /// Transact the given transaction.
    ///
    /// Internally sets transaction in context and use `replay` to execute the transaction.
    fn transact(&mut self, tx: Self::Tx) -> Result<(Self::Output, Self::State), Self::Error> {
        let output = self.transact_continue(tx)?;
        let state = self.finalize();
        Ok((output, state))
    }

    fn multi_transact(
        &mut self,
        txs: impl Iterator<Item = Self::Tx>,
    ) -> Result<Vec<Self::Output>, Self::Error> {
        let mut outputs = Vec::new();
        for tx in txs {
            outputs.push(self.transact_continue(tx)?);
        }
        Ok(outputs)
    }

    fn multi_transact_finalize(
        &mut self,
        txs: impl Iterator<Item = Self::Tx>,
    ) -> Result<(Vec<Self::Output>, Self::State), Self::Error> {
        let output = self.multi_transact(txs)?;
        let state = self.finalize();
        Ok((output, state))
    }

    fn clear_state(&mut self);
}

/// Extension of the [`ExecuteEvm`] trait that adds a method that commits the state after execution.
pub trait ExecuteCommitEvm: ExecuteEvm {
    /// Commit output of transaction execution.
    type CommitOutput;

    /// Transact the transaction and commit to the state.
    fn replay_commit(&mut self) -> Self::CommitOutput;

    /// Transact the transaction and commit to the state.
    fn transact_commit(&mut self, tx: Self::Tx) -> Self::CommitOutput {
        self.set_tx(tx);
        self.replay_commit()
    }
}

impl<CTX, INSP, INST, PRECOMPILES> ExecuteEvm for Evm<CTX, INSP, INST, PRECOMPILES>
where
    CTX: ContextTr<Journal: JournalTr<FinalOutput = JournalOutput>> + ContextSetters,
    INST: InstructionProvider<Context = CTX, InterpreterTypes = EthInterpreter>,
    PRECOMPILES: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    type Output = ExecutionResult<HaltReason>;
    type State = EvmState;
    type Error = EVMError<<CTX::Db as Database>::Error, InvalidTransaction>;

    type Tx = <CTX as ContextTr>::Tx;

    type Block = <CTX as ContextTr>::Block;

    fn transact_continue(&mut self, tx: Self::Tx) -> Result<Self::Output, Self::Error> {
        todo!("");
        // TODO run should return output without state.
        // let mut t = MainnetHandler::<_, _, EthFrame<_, _, _>>::default();
        // t.run(self)
    }

    fn finalize(&mut self) -> Self::State {
        todo!();
        //self.journal().finalize()
    }

    fn clear_state(&mut self) {
        todo!();
        //self.journal().revert_last()
    }

    fn set_tx(&mut self, tx: Self::Tx) {
        self.ctx.set_tx(tx);
    }

    fn set_block(&mut self, block: Self::Block) {
        self.ctx.set_block(block);
    }
}

impl<CTX, INSP, INST, PRECOMPILES> ExecuteCommitEvm for Evm<CTX, INSP, INST, PRECOMPILES>
where
    CTX: ContextTr<Journal: JournalTr<FinalOutput = JournalOutput>, Db: DatabaseCommit>
        + ContextSetters,
    INST: InstructionProvider<Context = CTX, InterpreterTypes = EthInterpreter>,
    PRECOMPILES: PrecompileProvider<CTX, Output = InterpreterResult>,
{
    type CommitOutput = Result<
        ExecutionResult<HaltReason>,
        EVMError<<CTX::Db as Database>::Error, InvalidTransaction>,
    >;

    fn replay_commit(&mut self) -> Self::CommitOutput {
        self.replay().map(|r| {
            self.db().commit(r.state);
            r.result
        })
    }
}
