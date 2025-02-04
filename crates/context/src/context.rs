pub mod performant_access;

use core::ops::{Deref, DerefMut};

use crate::{
    block::BlockEnv, cfg::CfgEnv, journaled_state::JournaledState, setters::ContextSetters,
    tx::TxEnv,
};
use auto_impl::auto_impl;
use context_interface::{
    block::BlockSetter, transaction::TransactionSetter, Block, BlockGetter, Cfg, CfgGetter,
    DatabaseGetter, ErrorGetter, Journal, JournalGetter, Transaction, TransactionGetter,
};
use database_interface::{Database, EmptyDB};
use derive_where::derive_where;
use interpreter::Host;
use primitives::U256;
use specification::hardfork::SpecId;

pub struct Evm<CTX, INSP, I, P> {
    pub ctx: Ctx<CTX, INSP>,
    pub enabled_inspection: bool,
    pub instruction: I,
    pub precompiles: P,
}

pub struct Ctx<CTX, INSP> {
    pub ctx: CTX,
    pub inspector: INSP,
}

impl<CTX> Evm<CTX, (), (), ()> {
    pub fn new(ctx: CTX) -> Self {
        Evm {
            ctx: Ctx { ctx, inspector: () },
            enabled_inspection: false,
            instruction: (),
            precompiles: (),
        }
    }
}

// }
// impl<CTX, FRAMECTX, INSP> MEVM<CTX, FRAMECTX, INSP> {
//     pub fn all_mut(&mut self) -> (&mut CTX, &mut FRAMECTX) {
//         (&mut self.ctx, &mut self.frame_ctx)
//     }

//     pub fn into_compontents(self) -> (CTX, FRAMECTX) {
//         (self.ctx, self.frame_ctx)
//     }

//     pub fn into_context(self) -> CTX {
//         self.ctx
//     }

//     pub fn ctx(&mut self) -> &mut CTX {
//         &mut self.ctx
//     }

//     pub fn frame_ctx(&mut self) -> &mut FRAMECTX {
//         &mut self.frame_ctx
//     }

//     pub fn modify_ctx<F, OCTX>(self, f: F) -> MEVM<OCTX, FRAMECTX, INSP>
//     where
//         F: FnOnce(CTX) -> OCTX,
//     {
//         MEVM {
//             ctx: f(self.ctx),
//             frame_ctx: self.frame_ctx,
//             inspector: self.inspector,
//         }
//     }
// }

impl<CTX: ContextSetters, INSP, I, P> ContextSetters for Evm<CTX, INSP, I, P> {
    type Tx = <CTX as ContextSetters>::Tx;
    type Block = <CTX as ContextSetters>::Block;

    fn set_tx(&mut self, tx: Self::Tx) {
        self.ctx.ctx.set_tx(tx);
    }

    fn set_block(&mut self, block: Self::Block) {
        self.ctx.ctx.set_block(block);
    }
}

impl<CTX, INSP, I, P> Deref for Evm<CTX, INSP, I, P> {
    type Target = CTX;

    fn deref(&self) -> &Self::Target {
        &self.ctx.ctx
    }
}

impl<CTX, INSP, I, P> DerefMut for Evm<CTX, INSP, I, P> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.ctx.ctx
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

    // Performant load of access list
    fn load_access_list(&mut self) -> Result<(), <Self::Db as Database>::Error>;
    // TODO do a default impls where access list from tx is completly copied and
    // loaded from journal.
}

impl<
        BLOCK: Block,
        TX: Transaction,
        DB: Database,
        CFG: Cfg,
        JOURNAL: Journal<Database = DB>,
        CHAIN,
    > ContextTrait for Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN>
{
    type Block = BLOCK;
    type Tx = TX;
    type Cfg = CFG;
    type Db = DB;
    type Journal = JOURNAL;
    type Chain = CHAIN;

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

    fn error(&mut self) -> &mut Result<(), <Self::Db as Database>::Error> {
        &mut self.error
    }

    fn load_access_list(&mut self) -> Result<(), <Self::Db as Database>::Error> {
        let journal = &mut self.journaled_state;
        let tx = &mut self.tx;
        let Some(access_list) = tx.access_list() else {
            return Ok(());
        };
        for access_list in access_list {
            journal.warm_account_and_storage(
                *access_list.0,
                access_list.1.iter().map(|i| U256::from_be_bytes(i.0)),
            )?;
        }
        Ok(())
    }
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
// /********    MEVEM block/tx setters/getters     *****/
// impl<CTX: BlockGetter, FRAMECTX, INSP> BlockGetter for MEVM<CTX, FRAMECTX, INSP> {
//     type Block = <CTX as BlockGetter>::Block;

//     fn block(&self) -> &Self::Block {
//         self.ctx.block()
//     }
// }

// impl<CTX: BlockSetter, FRAMECTX, INSP> BlockSetter for MEVM<CTX, FRAMECTX, INSP> {
//     fn set_block(&mut self, block: <Self as BlockGetter>::Block) {
//         self.ctx.set_block(block);
//     }
// }

// impl<CTX: TransactionGetter, FRAMECTX, INSP> TransactionGetter for MEVM<CTX, FRAMECTX, INSP> {
//     type Transaction = <CTX as TransactionGetter>::Transaction;

//     fn tx(&self) -> &Self::Transaction {
//         self.ctx.tx()
//     }
// }

// impl<CTX: TransactionSetter, FRAMECTX, INSP> TransactionSetter for MEVM<CTX, FRAMECTX, INSP> {
//     fn set_tx(&mut self, tx: <Self as TransactionGetter>::Transaction) {
//         self.ctx.set_tx(tx);
//     }
// }
