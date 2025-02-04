use context::Evm;
use context_interface::{block::BlockSetter, transaction::TransactionSetter};
use handler::{instructions::EthInstructionExecutor, EthPrecompileProvider};
use interpreter::interpreter::EthInterpreter;

pub trait MainBuilder: Sized {
    type Context;

    fn build_mainnet(
        self,
    ) -> Evm<
        Self::Context,
        (),
        EthInstructionExecutor<EthInterpreter, Self::Context>,
        EthPrecompileProvider<Self::Context>,
    >;
}

pub trait MainContext {
    fn mainnet() -> Self;
}

/// Execute EVM transactions.
//pub trait ExecuteEvm: BlockSetter + TransactionSetter {
pub trait ExecuteEvm {
    type Output;

    fn exec_previous(&mut self) -> Self::Output;

    // fn exec(&mut self, tx: Self::Transaction) -> Self::Output {
    //     self.set_tx(tx);
    //     self.exec_previous()
    // }
}

// /// Execute EVM transactions and commit to the state.
// pub trait ExecuteCommitEvm: ExecuteEvm {
//     type CommitOutput;

//     fn exec_commit_previous(&mut self) -> Self::CommitOutput;

//     fn exec_commit(&mut self, tx: Self::Transaction) -> Self::CommitOutput {
//         self.set_tx(tx);
//         self.exec_commit_previous()
//     }
// }
