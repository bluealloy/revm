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

pub trait InspectEvm: ExecuteEvm {
    type Inspector;

    fn set_inspector(&mut self, inspector: Self::Inspector);

    fn inspect_previous(&mut self) -> Self::Output;

    fn inspect_previous_with_inspector(&mut self, inspector: Self::Inspector) -> Self::Output {
        self.set_inspector(inspector);
        self.inspect_previous()
    }

    fn inspect_previous_with_tx(&mut self, tx: <Self as ContextSetters>::Tx) -> Self::Output {
        self.set_tx(tx);
        self.inspect_previous()
    }

    fn inspect(
        &mut self,
        tx: <Self as ContextSetters>::Tx,
        inspector: Self::Inspector,
    ) -> Self::Output {
        self.set_tx(tx);
        self.inspect_previous_with_inspector(inspector)
    }
}

pub trait InspectCommitEvm: InspectEvm + ExecuteCommitEvm {
    fn inspect_commit_previous(&mut self) -> Self::CommitOutput;

    fn inspect_commit_previous_with_inspector(
        &mut self,
        inspector: Self::Inspector,
    ) -> Self::CommitOutput {
        self.set_inspector(inspector);
        self.inspect_commit_previous()
    }

    fn inspect_commit(
        &mut self,
        tx: <Self as ContextSetters>::Tx,
        inspector: Self::Inspector,
    ) -> Self::CommitOutput {
        self.set_tx(tx);
        self.inspect_commit_previous_with_inspector(inspector)
    }
}
