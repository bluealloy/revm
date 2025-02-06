use crate::{
    Block, BlockGetter, Cfg, CfgGetter, Journal, JournalGetter, Transaction, TransactionGetter,
};
use database_interface::DatabaseGetter;

/// A dummy [Host] implementation.
#[derive(Clone, Debug, Default)]
pub struct DummyHost<BLOCK, TX, CFG, JOURNAL>
where
    BLOCK: Block,
    TX: Transaction,
    CFG: Cfg,
    JOURNAL: Journal,
{
    pub tx: TX,
    pub block: BLOCK,
    pub cfg: CFG,
    pub journal: JOURNAL,
}

impl<BLOCK, TX, CFG, JOURNAL> DummyHost<BLOCK, TX, CFG, JOURNAL>
where
    BLOCK: Block,
    TX: Transaction,
    CFG: Cfg + Default,
    JOURNAL: Journal + Default,
{
    /// Create a new dummy host with the given [`Transaction`] and [`Block`].
    #[inline]
    pub fn new(tx: TX, block: BLOCK) -> Self {
        Self {
            tx,
            block,
            cfg: CFG::default(),
            journal: JOURNAL::default(),
        }
    }

    /// Clears the storage and logs of the dummy host.
    #[inline]
    pub fn clear(&mut self) {
        self.journal.clear();
    }
}

impl<BLOCK: Block, TX: Transaction, CFG: Cfg, JOURNAL: Journal> BlockGetter
    for DummyHost<BLOCK, TX, CFG, JOURNAL>
{
    type Block = BLOCK;

    fn block(&self) -> &Self::Block {
        &self.block
    }
}

impl<BLOCK: Block, TX: Transaction, CFG: Cfg, JOURNAL: Journal> TransactionGetter
    for DummyHost<BLOCK, TX, CFG, JOURNAL>
{
    type Transaction = TX;

    fn tx(&self) -> &Self::Transaction {
        &self.tx
    }
}

impl<BLOCK: Block, TX: Transaction, CFG: Cfg, JOURNAL: Journal> CfgGetter
    for DummyHost<BLOCK, TX, CFG, JOURNAL>
{
    type Cfg = CFG;

    fn cfg(&self) -> &Self::Cfg {
        &self.cfg
    }
}

impl<BLOCK: Block, TX: Transaction, CFG: Cfg, JOURNAL: Journal> DatabaseGetter
    for DummyHost<BLOCK, TX, CFG, JOURNAL>
{
    type Database = <JOURNAL as Journal>::Database;

    fn db(&mut self) -> &mut Self::Database {
        self.journal.db()
    }

    fn db_ref(&self) -> &Self::Database {
        self.journal.db_ref()
    }
}

impl<BLOCK: Block, TX: Transaction, CFG: Cfg, JOURNAL: Journal> JournalGetter
    for DummyHost<BLOCK, TX, CFG, JOURNAL>
{
    type Journal = JOURNAL;

    fn journal(&mut self) -> &mut Self::Journal {
        &mut self.journal
    }

    fn journal_ref(&self) -> &Self::Journal {
        &self.journal
    }
}

// TODO(add dummy)
// impl<TX: Transaction, BLOCK: Block, CFG: Cfg, JOURNAL: Journal> Host
//     for DummyHost<BLOCK, TX, CFG, JOURNAL>
// {
//     #[inline]
//     fn set_error(
//         &mut self,
//         error: <<<Self as crate::JournalGetter>::Journal as crate::Journal>::Database as database_interface::Database>::Error,
//     ) {
//         panic!("Error: {:?}", error);
//     }
// }
