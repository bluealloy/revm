use handler::{ExecuteCommitEvm, ExecuteEvm};

/// InspectEvm is a API that allows inspecting the EVM.
///
/// It extends the `ExecuteEvm` trait and enabled setting inspector
///
pub trait InspectEvm: ExecuteEvm {
    type Inspector;

    /// Set the inspector for the EVM.
    ///
    /// this function is used to change inspector during execution.
    /// This function can't change Inspector type, changing inspector type can be done in
    /// `Evm` with `with_inspector` function.
    fn set_inspector(&mut self, inspector: Self::Inspector);

    /// Inspect the EVM with the current inspector and previous transaction.
    fn inspect_replay(&mut self) -> Self::Output;

    /// Inspect the EVM with the given inspector and transaction.
    fn inspect(&mut self, tx: Self::Tx, inspector: Self::Inspector) -> Self::Output {
        self.set_tx(tx);
        self.inspect_replay_with_inspector(inspector)
    }

    /// Inspect the EVM with the current inspector and previous transaction by replaying it.
    fn inspect_replay_with_inspector(&mut self, inspector: Self::Inspector) -> Self::Output {
        self.set_inspector(inspector);
        self.inspect_replay()
    }

    /// Inspect the EVM with the given transaction.
    fn inspect_with_tx(&mut self, tx: Self::Tx) -> Self::Output {
        self.set_tx(tx);
        self.inspect_replay()
    }
}

/// InspectCommitEvm is a API that allows inspecting similar to `InspectEvm` but it has
/// functions that commit the state diff to the database.
///
/// Functions return CommitOutput from [`ExecuteCommitEvm`] trait.
pub trait InspectCommitEvm: InspectEvm + ExecuteCommitEvm {
    /// Inspect the EVM with the current inspector and previous transaction, similar to [`InspectEvm::inspect_replay`]
    /// and commit the state diff to the database.
    fn inspect_replay_commit(&mut self) -> Self::CommitOutput;

    /// Inspects commit with the given inspector and previous transaction, similar to [`InspectEvm::inspect_replay_with_inspector`]
    /// and commit the state diff to the database.
    fn inspect_replay_commit_with_inspector(
        &mut self,
        inspector: Self::Inspector,
    ) -> Self::CommitOutput {
        self.set_inspector(inspector);
        self.inspect_replay_commit()
    }

    /// Inspect the EVM with the current inspector and previous transaction by replaying,similar to [`InspectEvm::inspect_replay_with_inspector`]
    /// and commit the state diff to the database.
    fn inspect_replay_with_inspector(&mut self, inspector: Self::Inspector) -> Self::CommitOutput {
        self.set_inspector(inspector);
        self.inspect_replay_commit()
    }

    /// Inspect the EVM with the given transaction and inspector similar to [`InspectEvm::inspect`]
    /// and commit the state diff to the database.
    fn inspect_commit(&mut self, tx: Self::Tx, inspector: Self::Inspector) -> Self::CommitOutput {
        self.set_tx(tx);
        self.inspect_replay_commit_with_inspector(inspector)
    }
}
