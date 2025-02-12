use context::setters::ContextSetters;

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
