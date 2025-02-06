use crate::Context;
use context_interface::{Block, Cfg, Database, Journal, Transaction};

pub trait ContextSetters {
    type Tx: Transaction;
    type Block: Block;

    fn set_tx(&mut self, tx: Self::Tx);
    fn set_block(&mut self, block: Self::Block);
}

impl<BLOCK, TX, CFG, DB, JOURNAL, CHAIN> ContextSetters
    for Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
where
    BLOCK: Block,
    TX: Transaction,
    CFG: Cfg,
    DB: Database,
    JOURNAL: Journal<Database = DB>,
{
    type Tx = TX;
    type Block = BLOCK;

    fn set_tx(&mut self, tx: Self::Tx) {
        self.tx = tx;
    }

    fn set_block(&mut self, block: Self::Block) {
        self.block = block;
    }
}
