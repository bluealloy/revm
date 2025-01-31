pub mod performant_access;

use core::ops::{Deref, DerefMut};

use crate::{block::BlockEnv, cfg::CfgEnv, journaled_state::JournaledState, tx::TxEnv};
use context_interface::{
    block::BlockSetter, transaction::TransactionSetter, Block, BlockGetter, Cfg, CfgGetter,
    DatabaseGetter, ErrorGetter, Journal, JournalGetter, Transaction, TransactionGetter,
};
use database_interface::{Database, EmptyDB};
use derive_where::derive_where;
use interpreter::Host;
use specification::hardfork::SpecId;

pub struct MEVM<CTX, FRAMECTX> {
    pub ctx: CTX,
    pub frame_ctx: FRAMECTX,
}

impl<CTX, FRAMECTX> MEVM<CTX, FRAMECTX> {
    pub fn all_mut(&mut self) -> (&mut CTX, &mut FRAMECTX) {
        (&mut self.ctx, &mut self.frame_ctx)
    }

    pub fn new(ctx: CTX, frame_ctx: FRAMECTX) -> Self {
        MEVM { ctx, frame_ctx }
    }

    pub fn into_compontents(self) -> (CTX, FRAMECTX) {
        (self.ctx, self.frame_ctx)
    }

    pub fn into_context(self) -> CTX {
        self.ctx
    }

    pub fn ctx(&mut self) -> &mut CTX {
        &mut self.ctx
    }

    pub fn frame_ctx(&mut self) -> &mut FRAMECTX {
        &mut self.frame_ctx
    }

    pub fn modify_ctx<F, OCTX>(self, f: F) -> MEVM<OCTX, FRAMECTX>
    where
        F: FnOnce(CTX) -> OCTX,
    {
        MEVM {
            ctx: f(self.ctx),
            frame_ctx: self.frame_ctx,
        }
    }
}

impl<CTX, FRAMECTX> Deref for MEVM<CTX, FRAMECTX> {
    type Target = CTX;

    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl<CTX, FRAMECTX> DerefMut for MEVM<CTX, FRAMECTX> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ctx
    }
}

/// EVM context contains data that EVM needs for execution.
#[derive_where(Clone, Debug; BLOCK, CFG, CHAIN, TX, DB, JOURNAL, <DB as Database>::Error)]
pub struct Context<
    BLOCK = BlockEnv,
    TX = TxEnv,
    CFG = CfgEnv,
    DB: Database = EmptyDB,
    JOURNAL: Journal<Database = DB> = JournaledState<DB>,
    CHAIN = (),
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
    /// Error that happened during execution.
    pub error: Result<(), <DB as Database>::Error>,
}

impl<
        BLOCK: Block + Default,
        TX: Transaction + Default,
        DB: Database,
        JOURNAL: Journal<Database = DB>,
        CHAIN: Default,
    > Context<BLOCK, TX, CfgEnv, DB, JOURNAL, CHAIN>
{
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
    JOURNAL: Journal<Database = DB>,
{
    pub fn with_new_journal<OJOURNAL: Journal<Database = DB>>(
        self,
        mut journal: OJOURNAL,
    ) -> Context<BLOCK, TX, CFG, DB, OJOURNAL, CHAIN> {
        journal.set_spec_id(self.cfg.spec().into());
        Context {
            tx: self.tx,
            block: self.block,
            cfg: self.cfg,
            journaled_state: journal,
            chain: self.chain,
            error: Ok(()),
        }
    }

    /// Creates a new context with a new database type.
    pub fn with_db<ODB: Database>(
        self,
        db: ODB,
    ) -> Context<BLOCK, TX, CFG, ODB, JournaledState<ODB>, CHAIN> {
        let spec = self.cfg.spec().into();
        let mut journaled_state = JournaledState::new(spec, db);
        journaled_state.set_spec_id(spec);
        Context {
            tx: self.tx,
            block: self.block,
            cfg: self.cfg,
            journaled_state,
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

    pub fn modify_tx<F>(&mut self, f: F)
    where
        F: FnOnce(&mut TX),
    {
        f(&mut self.tx);
    }

    pub fn modify_cfg<F>(&mut self, f: F)
    where
        F: FnOnce(&mut CFG),
    {
        f(&mut self.cfg);
        self.journaled_state.set_spec_id(self.cfg.spec().into());
    }

    pub fn modify_chain<F>(&mut self, f: F)
    where
        F: FnOnce(&mut CHAIN),
    {
        f(&mut self.chain);
    }

    pub fn modify_db<F>(&mut self, f: F)
    where
        F: FnOnce(&mut DB),
    {
        f(self.journaled_state.db());
    }

    pub fn modify_journal<F>(&mut self, f: F)
    where
        F: FnOnce(&mut JOURNAL),
    {
        f(&mut self.journaled_state);
    }
}

impl<BLOCK, TX, CFG, DB, JOURNAL, CHAIN> Host for Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
where
    BLOCK: Block,
    TX: Transaction,
    CFG: Cfg,
    DB: Database,
    JOURNAL: Journal<Database = DB>,
{
    fn set_error(
        &mut self,
        error: <<<Self as JournalGetter>::Journal as Journal>::Database as Database>::Error,
    ) {
        self.error = Err(error);
    }
}

impl<BLOCK, TX, CFG: Cfg, DB: Database, JOURNAL: Journal<Database = DB>, CHAIN> CfgGetter
    for Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
{
    type Cfg = CFG;

    fn cfg(&self) -> &Self::Cfg {
        &self.cfg
    }
}

impl<BLOCK, TX, SPEC, DB, JOURNAL, CHAIN> JournalGetter
    for Context<BLOCK, TX, SPEC, DB, JOURNAL, CHAIN>
where
    DB: Database,
    JOURNAL: Journal<Database = DB>,
{
    type Journal = JOURNAL;

    fn journal(&mut self) -> &mut Self::Journal {
        &mut self.journaled_state
    }

    fn journal_ref(&self) -> &Self::Journal {
        &self.journaled_state
    }
}

impl<BLOCK, TX, SPEC, DB, JOURNAL, CHAIN> DatabaseGetter
    for Context<BLOCK, TX, SPEC, DB, JOURNAL, CHAIN>
where
    DB: Database,
    JOURNAL: Journal<Database = DB>,
{
    type Database = DB;

    fn db(&mut self) -> &mut Self::Database {
        self.journaled_state.db()
    }

    fn db_ref(&self) -> &Self::Database {
        self.journaled_state.db_ref()
    }
}

impl<BLOCK, TX: Transaction, SPEC, DB: Database, JOURNAL: Journal<Database = DB>, CHAIN> ErrorGetter
    for Context<BLOCK, TX, SPEC, DB, JOURNAL, CHAIN>
{
    type Error = DB::Error;

    fn take_error(&mut self) -> Result<(), Self::Error> {
        core::mem::replace(&mut self.error, Ok(()))
    }
}

impl<BLOCK, TX: Transaction, SPEC, DB: Database, JOURNAL: Journal<Database = DB>, CHAIN>
    TransactionGetter for Context<BLOCK, TX, SPEC, DB, JOURNAL, CHAIN>
{
    type Transaction = TX;

    fn tx(&self) -> &Self::Transaction {
        &self.tx
    }
}

impl<BLOCK, TX: Transaction, SPEC, DB: Database, JOURNAL: Journal<Database = DB>, CHAIN>
    TransactionSetter for Context<BLOCK, TX, SPEC, DB, JOURNAL, CHAIN>
{
    fn set_tx(&mut self, tx: <Self as TransactionGetter>::Transaction) {
        self.tx = tx;
    }
}

impl<BLOCK: Block, TX, SPEC, DB: Database, JOURNAL: Journal<Database = DB>, CHAIN> BlockGetter
    for Context<BLOCK, TX, SPEC, DB, JOURNAL, CHAIN>
{
    type Block = BLOCK;

    fn block(&self) -> &Self::Block {
        &self.block
    }
}

impl<BLOCK: Block, TX, SPEC, DB: Database, JOURNAL: Journal<Database = DB>, CHAIN> BlockSetter
    for Context<BLOCK, TX, SPEC, DB, JOURNAL, CHAIN>
{
    fn set_block(&mut self, block: <Self as BlockGetter>::Block) {
        self.block = block;
    }
}
/********    MEVEM block/tx setters/getters     *****/

impl<CTX: BlockGetter, FRAMECTX> BlockGetter for MEVM<CTX, FRAMECTX> {
    type Block = <CTX as BlockGetter>::Block;

    fn block(&self) -> &Self::Block {
        self.ctx.block()
    }
}

impl<CTX: BlockSetter, FRAMECTX> BlockSetter for MEVM<CTX, FRAMECTX> {
    fn set_block(&mut self, block: <Self as BlockGetter>::Block) {
        self.ctx.set_block(block);
    }
}

impl<CTX: TransactionGetter, FRAMECTX> TransactionGetter for MEVM<CTX, FRAMECTX> {
    type Transaction = <CTX as TransactionGetter>::Transaction;

    fn tx(&self) -> &Self::Transaction {
        self.ctx.tx()
    }
}

impl<CTX: TransactionSetter, FRAMECTX> TransactionSetter for MEVM<CTX, FRAMECTX> {
    fn set_tx(&mut self, tx: <Self as TransactionGetter>::Transaction) {
        self.ctx.set_tx(tx);
    }
}
