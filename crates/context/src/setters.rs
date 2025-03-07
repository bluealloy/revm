use crate::Context;
use auto_impl::auto_impl;
use context_interface::{Block, Cfg, Database, JournalTr, Transaction};

/// Setters for the context.
#[auto_impl(&mut, Box)]
pub trait ContextSetters {
    /// Transaction type.
    type Tx: Transaction;
    /// Block type.
    type Block: Block;

    /// Set the transaction.
    fn set_tx(&mut self, tx: Self::Tx);

    /// Set the block.
    fn set_block(&mut self, block: Self::Block);
}

impl<BLOCK, TX, CFG, DB, JOURNAL, CHAIN> ContextSetters
    for Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
where
    BLOCK: Block,
    TX: Transaction,
    CFG: Cfg,
    DB: Database,
    JOURNAL: JournalTr<Database = DB>,
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
