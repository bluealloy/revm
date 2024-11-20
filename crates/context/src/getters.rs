use auto_impl::auto_impl;
use context_interface::{journaled_state::JournaledState, Block, Cfg, Transaction};
use database_interface::Database;

#[auto_impl(&, &mut, Box, Arc)]
pub trait CfgGetter {
    type Cfg: Cfg;

    fn cfg(&self) -> &Self::Cfg;
}

/// Helper that extracts database error from [`JournalStateGetter`].
pub type JournalStateGetterDBError<CTX> =
    <<<CTX as JournalStateGetter>::Journal as JournaledState>::Database as Database>::Error;

#[auto_impl(&mut, Box)]
pub trait JournalStateGetter {
    type Journal: JournaledState;

    fn journal(&mut self) -> &mut Self::Journal;
}

#[auto_impl(&mut, Box)]
pub trait DatabaseGetter {
    type Database: Database;

    fn db(&mut self) -> &mut Self::Database;
}

/// TODO change name of the trait
pub trait ErrorGetter {
    type Error;

    fn take_error(&mut self) -> Result<(), Self::Error>;
}

#[auto_impl(&, &mut, Box, Arc)]
pub trait TransactionGetter {
    type Transaction: Transaction;

    fn tx(&self) -> &Self::Transaction;
}

#[auto_impl(&, &mut, Box, Arc)]
pub trait BlockGetter {
    type Block: Block;

    fn block(&self) -> &Self::Block;
}
