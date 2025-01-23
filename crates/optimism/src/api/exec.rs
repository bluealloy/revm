use revm::context_interface::{
    block::BlockSetter, transaction::TransactionSetter, TransactionGetter,
};

/// Execute EVM transactions.
pub trait ExecuteOpEvm: BlockSetter + TransactionSetter {
    type Output;

    fn op_exec_previous(&mut self) -> Self::Output;

    fn op_exec(&mut self, tx: <Self as TransactionGetter>::Transaction) -> Self::Output {
        self.set_tx(tx);
        self.op_exec_previous()
    }
}

/// Execute EVM transactions and commit to the state.
pub trait ExecuteCommitOpEvm: ExecuteOpEvm {
    type CommitOutput;

    fn op_exec_commit_previous(&mut self) -> Self::CommitOutput;

    fn op_exec_commit(&mut self, tx: Self::Transaction) -> Self::CommitOutput {
        self.set_tx(tx);
        self.op_exec_commit_previous()
    }
}
