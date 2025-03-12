use context::setters::ContextSetters;
use handler::evm::{ExecuteCommitEvm, ExecuteEvm};

pub trait InspectEvm: ExecuteEvm {
    type Inspector;

    fn set_inspector(&mut self, inspector: Self::Inspector);

    fn inspect_replay(&mut self) -> Self::Output;

    fn inspect_replay_with_inspector(&mut self, inspector: Self::Inspector) -> Self::Output {
        self.set_inspector(inspector);
        self.inspect_replay()
    }

    fn inspect_replay_with_tx(&mut self, tx: <Self as ContextSetters>::Tx) -> Self::Output {
        self.set_tx(tx);
        self.inspect_replay()
    }

    fn inspect(
        &mut self,
        tx: <Self as ContextSetters>::Tx,
        inspector: Self::Inspector,
    ) -> Self::Output {
        self.set_tx(tx);
        self.inspect_replay_with_inspector(inspector)
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
