//! This module contains [`Context`] struct and implements [`ContextTr`] trait for it.
use crate::{block::BlockEnv, cfg::CfgEnv, journal::Journal, tx::TxEnv, LocalContext};
use context_interface::{
    context::{ContextError, ContextSetters},
    Block, Cfg, ContextTr, JournalTr, LocalContextTr, Transaction,
};
use database_interface::{Database, DatabaseRef, EmptyDB, WrapDatabaseRef};
use derive_where::derive_where;
use primitives::hardfork::SpecId;

/// EVM context contains data that EVM needs for execution.
#[derive_where(Clone, Debug; BLOCK, CFG, CHAIN, TX, DB, JOURNAL, <DB as Database>::Error, LOCAL)]
pub struct Context<
    BLOCK = BlockEnv,
    TX = TxEnv,
    CFG = CfgEnv,
    DB: Database = EmptyDB,
    JOURNAL: JournalTr<Database = DB> = Journal<DB>,
    CHAIN = (),
    LOCAL: LocalContextTr = LocalContext,
> {
    /// Block information.
    pub block: BLOCK,
    /// Transaction information.
    pub tx: TX,
    /// Configurations.
    pub cfg: CFG,
    /// EVM State with journaling support and database.
    pub journaled_state: JOURNAL,
    /// Inner context.
    pub chain: CHAIN,
    /// Local context that is filled by execution.
    pub local: LOCAL,
    /// Error that happened during execution.
    pub error: Result<(), ContextError<DB::Error>>,
}

impl<
        BLOCK: Block,
        TX: Transaction,
        DB: Database,
        CFG: Cfg,
        JOURNAL: JournalTr<Database = DB>,
        CHAIN,
        LOCAL: LocalContextTr,
    > ContextTr for Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN, LOCAL>
{
    type Block = BLOCK;
    type Tx = TX;
    type Cfg = CFG;
    type Db = DB;
    type Journal = JOURNAL;
    type Chain = CHAIN;
    type Local = LOCAL;

    fn tx(&self) -> &Self::Tx {
        &self.tx
    }

    fn block(&self) -> &Self::Block {
        &self.block
    }

    fn cfg(&self) -> &Self::Cfg {
        &self.cfg
    }

    fn journal(&mut self) -> &mut Self::Journal {
        &mut self.journaled_state
    }

    fn journal_ref(&self) -> &Self::Journal {
        &self.journaled_state
    }

    fn db(&mut self) -> &mut Self::Db {
        self.journaled_state.db()
    }

    fn db_ref(&self) -> &Self::Db {
        self.journaled_state.db_ref()
    }

    fn chain(&mut self) -> &mut Self::Chain {
        &mut self.chain
    }

    fn chain_ref(&self) -> &Self::Chain {
        &self.chain
    }

    fn local(&mut self) -> &mut Self::Local {
        &mut self.local
    }

    fn error(&mut self) -> &mut Result<(), ContextError<<Self::Db as Database>::Error>> {
        &mut self.error
    }

    fn tx_journal(&mut self) -> (&Self::Tx, &mut Self::Journal) {
        (&self.tx, &mut self.journaled_state)
    }

    fn tx_local(&mut self) -> (&Self::Tx, &mut Self::Local) {
        (&self.tx, &mut self.local)
    }
}

impl<
        BLOCK: Block,
        TX: Transaction,
        DB: Database,
        CFG: Cfg,
        JOURNAL: JournalTr<Database = DB>,
        CHAIN,
    > ContextSetters for Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
{
    fn set_tx(&mut self, tx: Self::Tx) {
        self.tx = tx;
    }

    fn set_block(&mut self, block: Self::Block) {
        self.block = block;
    }
}

impl<
        BLOCK: Block + Default,
        TX: Transaction + Default,
        DB: Database,
        JOURNAL: JournalTr<Database = DB>,
        CHAIN: Default,
    > Context<BLOCK, TX, CfgEnv, DB, JOURNAL, CHAIN>
{
    /// Creates a new context with a new database type.
    ///
    /// This will create a new [`Journal`] object.
    pub fn new(db: DB, spec: SpecId) -> Self {
        let mut journaled_state = JOURNAL::new(db);
        journaled_state.set_spec_id(spec);
        Self {
            tx: TX::default(),
            block: BLOCK::default(),
            cfg: CfgEnv {
                spec,
                ..Default::default()
            },
            local: LocalContext::default(),
            journaled_state,
            chain: Default::default(),
            error: Ok(()),
        }
    }
}

impl<BLOCK, TX, CFG, DB, JOURNAL, CHAIN> Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
where
    BLOCK: Block,
    TX: Transaction,
    CFG: Cfg,
    DB: Database,
    JOURNAL: JournalTr<Database = DB>,
{
    /// Creates a new context with a new journal type. New journal needs to have the same database type.
    pub fn with_new_journal<OJOURNAL: JournalTr<Database = DB>>(
        self,
        mut journal: OJOURNAL,
    ) -> Context<BLOCK, TX, CFG, DB, OJOURNAL, CHAIN> {
        journal.set_spec_id(self.cfg.spec().into());
        Context {
            tx: self.tx,
            block: self.block,
            cfg: self.cfg,
            journaled_state: journal,
            local: self.local,
            chain: self.chain,
            error: Ok(()),
        }
    }

    /// Creates a new context with a new database type.
    ///
    /// This will create a new [`Journal`] object.
    pub fn with_db<ODB: Database>(
        self,
        db: ODB,
    ) -> Context<BLOCK, TX, CFG, ODB, Journal<ODB>, CHAIN> {
        let spec = self.cfg.spec().into();
        let mut journaled_state = Journal::new(db);
        journaled_state.set_spec_id(spec);
        Context {
            tx: self.tx,
            block: self.block,
            cfg: self.cfg,
            journaled_state,
            local: self.local,
            chain: self.chain,
            error: Ok(()),
        }
    }

    /// Creates a new context with a new `DatabaseRef` type.
    pub fn with_ref_db<ODB: DatabaseRef>(
        self,
        db: ODB,
    ) -> Context<BLOCK, TX, CFG, WrapDatabaseRef<ODB>, Journal<WrapDatabaseRef<ODB>>, CHAIN> {
        let spec = self.cfg.spec().into();
        let mut journaled_state = Journal::new(WrapDatabaseRef(db));
        journaled_state.set_spec_id(spec);
        Context {
            tx: self.tx,
            block: self.block,
            cfg: self.cfg,
            journaled_state,
            local: self.local,
            chain: self.chain,
            error: Ok(()),
        }
    }

    /// Creates a new context with a new block type.
    pub fn with_block<OB: Block>(self, block: OB) -> Context<OB, TX, CFG, DB, JOURNAL, CHAIN> {
        Context {
            tx: self.tx,
            block,
            cfg: self.cfg,
            journaled_state: self.journaled_state,
            local: self.local,
            chain: self.chain,
            error: Ok(()),
        }
    }
    /// Creates a new context with a new transaction type.
    pub fn with_tx<OTX: Transaction>(
        self,
        tx: OTX,
    ) -> Context<BLOCK, OTX, CFG, DB, JOURNAL, CHAIN> {
        Context {
            tx,
            block: self.block,
            cfg: self.cfg,
            journaled_state: self.journaled_state,
            local: self.local,
            chain: self.chain,
            error: Ok(()),
        }
    }

    /// Creates a new context with a new chain type.
    pub fn with_chain<OC>(self, chain: OC) -> Context<BLOCK, TX, CFG, DB, JOURNAL, OC> {
        Context {
            tx: self.tx,
            block: self.block,
            cfg: self.cfg,
            journaled_state: self.journaled_state,
            local: self.local,
            chain,
            error: Ok(()),
        }
    }

    /// Creates a new context with a new chain type.
    pub fn with_cfg<OCFG: Cfg>(
        mut self,
        cfg: OCFG,
    ) -> Context<BLOCK, TX, OCFG, DB, JOURNAL, CHAIN> {
        self.journaled_state.set_spec_id(cfg.spec().into());
        Context {
            tx: self.tx,
            block: self.block,
            cfg,
            journaled_state: self.journaled_state,
            local: self.local,
            chain: self.chain,
            error: Ok(()),
        }
    }

    /// Modifies the context configuration.
    #[must_use]
    pub fn modify_cfg_chained<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut CFG),
    {
        f(&mut self.cfg);
        self.journaled_state.set_spec_id(self.cfg.spec().into());
        self
    }

    /// Modifies the context block.
    #[must_use]
    pub fn modify_block_chained<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut BLOCK),
    {
        self.modify_block(f);
        self
    }

    /// Modifies the context transaction.
    #[must_use]
    pub fn modify_tx_chained<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut TX),
    {
        self.modify_tx(f);
        self
    }

    /// Modifies the context chain.
    #[must_use]
    pub fn modify_chain_chained<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut CHAIN),
    {
        self.modify_chain(f);
        self
    }

    /// Modifies the context database.
    #[must_use]
    pub fn modify_db_chained<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut DB),
    {
        self.modify_db(f);
        self
    }

    /// Modifies the context journal.
    #[must_use]
    pub fn modify_journal_chained<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut JOURNAL),
    {
        self.modify_journal(f);
        self
    }

    /// Modifies the context block.
    pub fn modify_block<F>(&mut self, f: F)
    where
        F: FnOnce(&mut BLOCK),
    {
        f(&mut self.block);
    }

    /// Modifies the context transaction.
    pub fn modify_tx<F>(&mut self, f: F)
    where
        F: FnOnce(&mut TX),
    {
        f(&mut self.tx);
    }

    /// Modifies the context configuration.
    pub fn modify_cfg<F>(&mut self, f: F)
    where
        F: FnOnce(&mut CFG),
    {
        f(&mut self.cfg);
        self.journaled_state.set_spec_id(self.cfg.spec().into());
    }

    /// Modifies the context chain.
    pub fn modify_chain<F>(&mut self, f: F)
    where
        F: FnOnce(&mut CHAIN),
    {
        f(&mut self.chain);
    }

    /// Modifies the context database.
    pub fn modify_db<F>(&mut self, f: F)
    where
        F: FnOnce(&mut DB),
    {
        f(self.journaled_state.db());
    }

    /// Modifies the context journal.
    pub fn modify_journal<F>(&mut self, f: F)
    where
        F: FnOnce(&mut JOURNAL),
    {
        f(&mut self.journaled_state);
    }
}
