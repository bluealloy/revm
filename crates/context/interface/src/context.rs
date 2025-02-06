use crate::{Block, Cfg, Database, Journal, Transaction};
use auto_impl::auto_impl;

#[auto_impl(&mut, Box)]
pub trait ContextTrait {
    type Block: Block;
    type Tx: Transaction;
    type Cfg: Cfg;
    type Db: Database;
    type Journal: Journal<Database = Self::Db>;
    type Chain;

    fn tx(&self) -> &Self::Tx;
    fn block(&self) -> &Self::Block;
    fn cfg(&self) -> &Self::Cfg;
    fn journal(&mut self) -> &mut Self::Journal;
    fn journal_ref(&self) -> &Self::Journal;
    fn db(&mut self) -> &mut Self::Db;
    fn db_ref(&self) -> &Self::Db;
    fn chain(&mut self) -> &mut Self::Chain;
    fn error(&mut self) -> &mut Result<(), <Self::Db as Database>::Error>;
    fn tx_journal(&mut self) -> (&mut Self::Tx, &mut Self::Journal);
}
