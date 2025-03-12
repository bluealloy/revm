use context::setters::ContextSetters;
use handler::evm::{ExecuteCommitEvm, ExecuteEvm};

pub trait InspectEvm: ExecuteEvm {
    type Inspector;

    fn set_inspector(&mut self, inspector: Self::Inspector);

    fn inspect_replay(&mut self) -> Self::Output;

    fn inspect_with_inspector(&mut self, inspector: Self::Inspector) -> Self::Output {
        self.set_inspector(inspector);
        self.inspect_replay()
    }
}

pub fn inspect_with_tx<EVM: ContextSetters + InspectEvm>(
    evm: &mut EVM,
    tx: EVM::Tx,
) -> EVM::Output {
    evm.set_tx(tx);
    evm.inspect_replay()
}

pub fn inspect<EVM: ContextSetters + InspectEvm>(
    evm: &mut EVM,
    tx: EVM::Tx,
    inspector: EVM::Inspector,
) -> EVM::Output {
    evm.set_tx(tx);
    evm.inspect_with_inspector(inspector)
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
}

pub fn inspect_commit<EVM: ContextSetters + InspectCommitEvm>(
    evm: &mut EVM,
    tx: EVM::Tx,
    inspector: EVM::Inspector,
) -> EVM::CommitOutput {
    evm.set_tx(tx);
    evm.inspect_commit_previous_with_inspector(inspector)
}
