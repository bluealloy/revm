use context_interface::{block::BlockSetter, transaction::TransactionSetter};

/// Execute EVM transactions.
pub trait ExecuteEvm: BlockSetter + TransactionSetter {
    type Output;

    fn exec_previous_tx(&mut self) -> Self::Output;

    fn exec(&mut self, tx: Self::Transaction) -> Self::Output {
        self.set_tx(tx);
        self.exec_previous_tx()
    }
}

/// Execute EVM transactions and commit to the state.
pub trait ExecuteCommitEvm: ExecuteEvm {
    type CommitOutput;

    fn exec_commit_previous_tx(&mut self) -> Self::CommitOutput;

    fn exec_commit(&mut self, tx: Self::Transaction) -> Self::CommitOutput {
        self.set_tx(tx);
        self.exec_commit_previous_tx()
    }
}
