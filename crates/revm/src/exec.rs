use context_interface::{Block, Transaction};

pub trait EvmExec {
    type TX: Transaction;
    type BLOCK: Block;

    fn set_block(&mut self, block: Self::BLOCK);

    fn transact(&mut self, tx: Self::TX);
}
