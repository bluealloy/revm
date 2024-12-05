use context_interface::{Block, Transaction};

pub trait EvmExec {
    type Transaction: Transaction;
    type Block: Block;
    type Output;

    fn set_block(&mut self, block: Self::Block);

    fn set_tx(&mut self, tx: Self::Transaction);

    fn exec(&mut self) -> Self::Output;
}

pub trait EvmCommit: EvmExec {
    type CommitOutput;

    fn exec_commit(&mut self) -> Self::CommitOutput;
}
