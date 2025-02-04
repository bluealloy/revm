use context::{setters::ContextSetters, Evm};
use handler::{inspector, instructions::EthInstructions, noop::NoOpInspector, EthPrecompiles};
use interpreter::interpreter::EthInterpreter;

pub trait MainBuilder: Sized {
    type Context;

    fn build_mainnet(
        self,
    ) -> Evm<
        Self::Context,
        NoOpInspector,
        EthInstructions<EthInterpreter, Self::Context>,
        EthPrecompiles<Self::Context>,
    >;

    fn build_mainnet_with_inspector<INSP>(
        self,
        inspector: INSP,
    ) -> Evm<
        Self::Context,
        INSP,
        EthInstructions<EthInterpreter, Self::Context>,
        EthPrecompiles<Self::Context>,
    >;
}

/// Trait used to initialize Context with default mainnet types.
pub trait MainContext {
    fn mainnet() -> Self;
}

/// Execute EVM transactions.
//pub trait ExecuteEvm: BlockSetter + TransactionSetter {
pub trait ExecuteEvm: ContextSetters {
    type Output;

    fn exec_previous(&mut self) -> Self::Output;

    fn exec(&mut self, tx: Self::Tx) -> Self::Output {
        self.set_tx(tx);
        self.exec_previous()
    }
}

/// Execute EVM transactions and commit to the state.
pub trait ExecuteCommitEvm: ExecuteEvm {
    type CommitOutput;

    fn exec_commit_previous(&mut self) -> Self::CommitOutput;

    fn exec_commit(&mut self, tx: Self::Tx) -> Self::CommitOutput {
        self.set_tx(tx);
        self.exec_commit_previous()
    }
}
