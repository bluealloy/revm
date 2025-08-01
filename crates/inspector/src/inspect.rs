use context::result::ExecResultAndState;
use handler::{system_call::SYSTEM_ADDRESS, ExecuteCommitEvm, ExecuteEvm, SystemCallEvm};
use primitives::{Address, Bytes};

/// InspectEvm is a API that allows inspecting the EVM.
///
/// It extends the `ExecuteEvm` trait and enabled setting inspector
///
pub trait InspectEvm: ExecuteEvm {
    /// The inspector type used for inspecting EVM execution.
    type Inspector;

    /// Set the inspector for the EVM.
    ///
    /// this function is used to change inspector during execution.
    /// This function can't change Inspector type, changing inspector type can be done in
    /// `Evm` with `with_inspector` function.
    fn set_inspector(&mut self, inspector: Self::Inspector);

    /// Inspect the EVM with the given transaction.
    fn inspect_one_tx(&mut self, tx: Self::Tx) -> Result<Self::ExecutionResult, Self::Error>;

    /// Inspect the EVM and finalize the state.
    fn inspect_tx(
        &mut self,
        tx: Self::Tx,
    ) -> Result<ExecResultAndState<Self::ExecutionResult, Self::State>, Self::Error> {
        let output = self.inspect_one_tx(tx)?;
        let state = self.finalize();
        Ok(ExecResultAndState::new(output, state))
    }

    /// Inspect the EVM with the given inspector and transaction, and finalize the state.
    fn inspect(
        &mut self,
        tx: Self::Tx,
        inspector: Self::Inspector,
    ) -> Result<ExecResultAndState<Self::ExecutionResult, Self::State>, Self::Error> {
        let output = self.inspect_one(tx, inspector)?;
        let state = self.finalize();
        Ok(ExecResultAndState::new(output, state))
    }

    /// Inspect the EVM with the given inspector and transaction.
    fn inspect_one(
        &mut self,
        tx: Self::Tx,
        inspector: Self::Inspector,
    ) -> Result<Self::ExecutionResult, Self::Error> {
        self.set_inspector(inspector);
        self.inspect_one_tx(tx)
    }
}

/// InspectCommitEvm is a API that allows inspecting similar to `InspectEvm` but it has
/// functions that commit the state diff to the database.
///
/// Functions return CommitOutput from [`ExecuteCommitEvm`] trait.
pub trait InspectCommitEvm: InspectEvm + ExecuteCommitEvm {
    /// Inspect the EVM with the current inspector and previous transaction by replaying, similar to [`InspectEvm::inspect_tx`]
    /// and commit the state diff to the database.
    fn inspect_tx_commit(&mut self, tx: Self::Tx) -> Result<Self::ExecutionResult, Self::Error> {
        let output = self.inspect_one_tx(tx)?;
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
        let output = self.inspect_one(tx, inspector)?;
        self.commit_inner();
        Ok(output)
    }
}

/// InspectSystemCallEvm is an API that allows inspecting system calls in the EVM.
///
/// It extends [`InspectEvm`] and [`SystemCallEvm`] traits to provide inspection
/// capabilities for system transactions, enabling tracing and debugging of
/// system calls similar to regular transactions.
pub trait InspectSystemCallEvm: InspectEvm + SystemCallEvm {
    /// Inspect a system call with the current inspector.
    ///
    /// Similar to [`InspectEvm::inspect_one_tx`] but for system calls.
    /// Uses [`SYSTEM_ADDRESS`] as the caller.
    fn inspect_one_system_call(
        &mut self,
        system_contract_address: Address,
        data: Bytes,
    ) -> Result<Self::ExecutionResult, Self::Error> {
        self.inspect_one_system_call_with_caller(SYSTEM_ADDRESS, system_contract_address, data)
    }

    /// Inspect a system call with the current inspector and a custom caller.
    ///
    /// Similar to [`InspectEvm::inspect_one_tx`] but for system calls with a custom caller.
    fn inspect_one_system_call_with_caller(
        &mut self,
        caller: Address,
        system_contract_address: Address,
        data: Bytes,
    ) -> Result<Self::ExecutionResult, Self::Error>;

    /// Inspect a system call and finalize the state.
    ///
    /// Similar to [`InspectEvm::inspect_tx`] but for system calls.
    fn inspect_system_call(
        &mut self,
        system_contract_address: Address,
        data: Bytes,
    ) -> Result<ExecResultAndState<Self::ExecutionResult, Self::State>, Self::Error> {
        let output = self.inspect_one_system_call(system_contract_address, data)?;
        let state = self.finalize();
        Ok(ExecResultAndState::new(output, state))
    }

    /// Inspect a system call with a custom caller and finalize the state.
    ///
    /// Similar to [`InspectEvm::inspect_tx`] but for system calls with a custom caller.
    fn inspect_system_call_with_caller(
        &mut self,
        caller: Address,
        system_contract_address: Address,
        data: Bytes,
    ) -> Result<ExecResultAndState<Self::ExecutionResult, Self::State>, Self::Error> {
        let output =
            self.inspect_one_system_call_with_caller(caller, system_contract_address, data)?;
        let state = self.finalize();
        Ok(ExecResultAndState::new(output, state))
    }

    /// Inspect a system call with a given inspector.
    ///
    /// Similar to [`InspectEvm::inspect_one`] but for system calls.
    fn inspect_one_system_call_with_inspector(
        &mut self,
        system_contract_address: Address,
        data: Bytes,
        inspector: Self::Inspector,
    ) -> Result<Self::ExecutionResult, Self::Error> {
        self.set_inspector(inspector);
        self.inspect_one_system_call(system_contract_address, data)
    }

    /// Inspect a system call with a given inspector and finalize the state.
    ///
    /// Similar to [`InspectEvm::inspect`] but for system calls.
    fn inspect_system_call_with_inspector(
        &mut self,
        system_contract_address: Address,
        data: Bytes,
        inspector: Self::Inspector,
    ) -> Result<ExecResultAndState<Self::ExecutionResult, Self::State>, Self::Error> {
        let output =
            self.inspect_one_system_call_with_inspector(system_contract_address, data, inspector)?;
        let state = self.finalize();
        Ok(ExecResultAndState::new(output, state))
    }

    /// Inspect a system call with a given inspector and caller.
    ///
    /// Similar to [`InspectEvm::inspect_one`] but for system calls.
    fn inspect_one_system_call_with_inspector_and_caller(
        &mut self,
        caller: Address,
        system_contract_address: Address,
        data: Bytes,
        inspector: Self::Inspector,
    ) -> Result<Self::ExecutionResult, Self::Error> {
        self.set_inspector(inspector);
        self.inspect_one_system_call_with_caller(caller, system_contract_address, data)
    }

    /// Inspect a system call with a given inspector and finalize the state.
    ///
    /// Similar to [`InspectEvm::inspect`] but for system calls.
    fn inspect_system_call_with_inspector_and_caller(
        &mut self,
        caller: Address,
        system_contract_address: Address,
        data: Bytes,
        inspector: Self::Inspector,
    ) -> Result<ExecResultAndState<Self::ExecutionResult, Self::State>, Self::Error> {
        let output = self.inspect_one_system_call_with_inspector_and_caller(
            caller,
            system_contract_address,
            data,
            inspector,
        )?;
        let state = self.finalize();
        Ok(ExecResultAndState::new(output, state))
    }
}
