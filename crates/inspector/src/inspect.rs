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

    /// Inspect the EVM with the given transaction.
    fn inspect_with_tx(&mut self, tx: Self::Tx) -> Result<Self::ExecutionResult, Self::Error>;

    /// Inspect the EVM with the given inspector and transaction.
    fn inspect(
        &mut self,
        tx: Self::Tx,
        inspector: Self::Inspector,
    ) -> Result<Self::ExecutionResult, Self::Error> {
        self.set_inspector(inspector);
        self.inspect_with_tx(tx)
    }

    /// Inspect the EVM
    fn inspect_with_tx_finalize(
        &mut self,
        tx: Self::Tx,
    ) -> Result<(Self::ExecutionResult, Self::State), Self::Error> {
        let output = self.inspect_with_tx(tx)?;
        let state = self.finalize();
        Ok((output, state))
    }
}

/// InspectCommitEvm is a API that allows inspecting similar to `InspectEvm` but it has
/// functions that commit the state diff to the database.
///
/// Functions return CommitOutput from [`ExecuteCommitEvm`] trait.
pub trait InspectCommitEvm: InspectEvm + ExecuteCommitEvm {
    /// Inspect the EVM with the current inspector and previous transaction by replaying,similar to [`InspectEvm::inspect_with_tx`]
    /// and commit the state diff to the database.
    fn inspect_with_tx_commit(
        &mut self,
        tx: Self::Tx,
    ) -> Result<Self::ExecutionResult, Self::Error> {
        let output = self.inspect_with_tx(tx)?;
        self.commit_inner();
        Ok(output)
    }

    /// Inspect the EVM with the given transaction and inspector similar to [`InspectEvm::inspect`]
    /// and commit the state diff to the database.
    fn inspect_commit(
        &mut self,
        tx: Self::Tx,
        inspector: Self::Inspector,
    ) -> Result<Self::ExecutionResult, Self::Error> {
        let output = self.inspect(tx, inspector)?;
        self.commit_inner();
        Ok(output)
    }
}
