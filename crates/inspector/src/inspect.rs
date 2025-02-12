use revm::{context::setters::ContextSetters, ExecuteCommitEvm, ExecuteEvm};

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
