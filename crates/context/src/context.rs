//! This module contains [`Context`] struct and implements [`ContextTr`] trait for it.
use crate::{block::BlockEnv, cfg::CfgEnv, journal::Journal, tx::TxEnv, LocalContext};
use context_interface::{
    context::{ContextError, ContextSetters, SStoreResult, SelfDestructResult, StateLoad},
    journaled_state::AccountLoad,
    Block, Cfg, ContextTr, Host, JournalTr, LocalContextTr, Transaction, TransactionType,
};
use database_interface::{Database, DatabaseRef, EmptyDB, WrapDatabaseRef};
use derive_where::derive_where;
use primitives::{hardfork::SpecId, Address, Bytes, Log, StorageKey, StorageValue, B256, U256};

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

    #[inline]
    fn tx(&self) -> &Self::Tx {
        &self.tx
    }

    #[inline]
    fn block(&self) -> &Self::Block {
        &self.block
    }

    #[inline]
    fn cfg(&self) -> &Self::Cfg {
        &self.cfg
    }

    #[inline]
    fn journal(&self) -> &Self::Journal {
        &self.journaled_state
    }

    #[inline]
    fn journal_mut(&mut self) -> &mut Self::Journal {
        &mut self.journaled_state
    }

    #[inline]
    fn journal_ref(&self) -> &Self::Journal {
        &self.journaled_state
    }

    #[inline]
    fn db(&self) -> &Self::Db {
        self.journaled_state.db()
    }

    #[inline]
    fn db_mut(&mut self) -> &mut Self::Db {
        self.journaled_state.db_mut()
    }

    #[inline]
    fn chain(&self) -> &Self::Chain {
        &self.chain
    }

    #[inline]
    fn chain_mut(&mut self) -> &mut Self::Chain {
        &mut self.chain
    }

    #[inline]
    fn local(&self) -> &Self::Local {
        &self.local
    }

    #[inline]
    fn local_mut(&mut self) -> &mut Self::Local {
        &mut self.local
    }

    #[inline]
    fn error(&mut self) -> &mut Result<(), ContextError<<Self::Db as Database>::Error>> {
        &mut self.error
    }

    #[inline]
    fn tx_journal_mut(&mut self) -> (&Self::Tx, &mut Self::Journal) {
        (&self.tx, &mut self.journaled_state)
    }

    #[inline]
    fn tx_local_mut(&mut self) -> (&Self::Tx, &mut Self::Local) {
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
        LOCAL: LocalContextTr,
    > ContextSetters for Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN, LOCAL>
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
        LOCAL: LocalContextTr + Default,
    > Context<BLOCK, TX, CfgEnv, DB, JOURNAL, CHAIN, LOCAL>
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
            local: LOCAL::default(),
            journaled_state,
            chain: Default::default(),
            error: Ok(()),
        }
    }
}

impl<BLOCK, TX, CFG, DB, JOURNAL, CHAIN, LOCAL> Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN, LOCAL>
where
    BLOCK: Block,
    TX: Transaction,
    CFG: Cfg,
    DB: Database,
    JOURNAL: JournalTr<Database = DB>,
    LOCAL: LocalContextTr,
{
    /// Creates a new context with a new journal type. New journal needs to have the same database type.
    pub fn with_new_journal<OJOURNAL: JournalTr<Database = DB>>(
        self,
        mut journal: OJOURNAL,
    ) -> Context<BLOCK, TX, CFG, DB, OJOURNAL, CHAIN, LOCAL> {
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
    ) -> Context<BLOCK, TX, CFG, ODB, Journal<ODB>, CHAIN, LOCAL> {
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
    ) -> Context<BLOCK, TX, CFG, WrapDatabaseRef<ODB>, Journal<WrapDatabaseRef<ODB>>, CHAIN, LOCAL>
    {
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
    pub fn with_block<OB: Block>(
        self,
        block: OB,
    ) -> Context<OB, TX, CFG, DB, JOURNAL, CHAIN, LOCAL> {
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
    ) -> Context<BLOCK, OTX, CFG, DB, JOURNAL, CHAIN, LOCAL> {
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
    pub fn with_chain<OC>(self, chain: OC) -> Context<BLOCK, TX, CFG, DB, JOURNAL, OC, LOCAL> {
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
    ) -> Context<BLOCK, TX, OCFG, DB, JOURNAL, CHAIN, LOCAL> {
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

    /// Creates a new context with a new local context type.
    pub fn with_local<OL: LocalContextTr>(
        self,
        local: OL,
    ) -> Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN, OL> {
        Context {
            tx: self.tx,
            block: self.block,
            cfg: self.cfg,
            journaled_state: self.journaled_state,
            local,
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
        f(self.journaled_state.db_mut());
    }

    /// Modifies the context journal.
    pub fn modify_journal<F>(&mut self, f: F)
    where
        F: FnOnce(&mut JOURNAL),
    {
        f(&mut self.journaled_state);
    }

    /// Modifies the local context.
    pub fn modify_local<F>(&mut self, f: F)
    where
        F: FnOnce(&mut LOCAL),
    {
        f(&mut self.local);
    }
}

impl<
        BLOCK: Block,
        TX: Transaction,
        CFG: Cfg,
        DB: Database,
        JOURNAL: JournalTr<Database = DB>,
        CHAIN,
        LOCAL: LocalContextTr,
    > Host for Context<BLOCK, TX, CFG, DB, JOURNAL, CHAIN, LOCAL>
{
    /* Block */

    fn basefee(&self) -> U256 {
        U256::from(self.block().basefee())
    }

    fn blob_gasprice(&self) -> U256 {
        U256::from(self.block().blob_gasprice().unwrap_or(0))
    }

    fn gas_limit(&self) -> U256 {
        U256::from(self.block().gas_limit())
    }

    fn difficulty(&self) -> U256 {
        self.block().difficulty()
    }

    fn prevrandao(&self) -> Option<U256> {
        self.block().prevrandao().map(|r| r.into())
    }

    fn block_number(&self) -> U256 {
        self.block().number()
    }

    fn timestamp(&self) -> U256 {
        U256::from(self.block().timestamp())
    }

    fn beneficiary(&self) -> Address {
        self.block().beneficiary()
    }

    fn chain_id(&self) -> U256 {
        U256::from(self.cfg().chain_id())
    }

    /* Transaction */

    fn effective_gas_price(&self) -> U256 {
        let basefee = self.block().basefee();
        U256::from(self.tx().effective_gas_price(basefee as u128))
    }

    fn caller(&self) -> Address {
        self.tx().caller()
    }

    fn blob_hash(&self, number: usize) -> Option<U256> {
        self.tx()
            .blob_versioned_hashes()
            .get(number)
            .map(|t| U256::from_be_bytes(t.0))
    }

    /* Config */

    fn max_initcode_size(&self) -> usize {
        self.cfg().max_initcode_size()
    }

    /* Database */

    fn block_hash(&mut self, requested_number: u64) -> Option<B256> {
        self.db_mut()
            .block_hash(requested_number)
            .map_err(|e| {
                *self.error() = Err(e.into());
            })
            .ok()
    }

    /* Journal */

    fn load_account_delegated(&mut self, address: Address) -> Option<StateLoad<AccountLoad>> {
        self.journal_mut()
            .load_account_delegated(address)
            .map_err(|e| {
                *self.error() = Err(e.into());
            })
            .ok()
    }

    /// Gets balance of `address` and if the account is cold.
    fn balance(&mut self, address: Address) -> Option<StateLoad<U256>> {
        self.journal_mut()
            .load_account(address)
            .map(|acc| acc.map(|a| a.info.balance))
            .map_err(|e| {
                *self.error() = Err(e.into());
            })
            .ok()
    }

    /// Gets code of `address` and if the account is cold.
    fn load_account_code(&mut self, address: Address) -> Option<StateLoad<Bytes>> {
        self.journal_mut()
            .code(address)
            .map_err(|e| {
                *self.error() = Err(e.into());
            })
            .ok()
    }

    /// Gets code hash of `address` and if the account is cold.
    fn load_account_code_hash(&mut self, address: Address) -> Option<StateLoad<B256>> {
        self.journal_mut()
            .code_hash(address)
            .map_err(|e| {
                *self.error() = Err(e.into());
            })
            .ok()
    }

    /// Gets storage value of `address` at `index` and if the account is cold.
    fn sload(&mut self, address: Address, index: StorageKey) -> Option<StateLoad<StorageValue>> {
        self.journal_mut()
            .sload(address, index)
            .map_err(|e| {
                *self.error() = Err(e.into());
            })
            .ok()
    }

    /// Sets storage value of account address at index.
    ///
    /// Returns [`StateLoad`] with [`SStoreResult`] that contains original/new/old storage value.
    fn sstore(
        &mut self,
        address: Address,
        index: StorageKey,
        value: StorageValue,
    ) -> Option<StateLoad<SStoreResult>> {
        self.journal_mut()
            .sstore(address, index, value)
            .map_err(|e| {
                *self.error() = Err(e.into());
            })
            .ok()
    }

    /// Gets the transient storage value of `address` at `index`.
    fn tload(&mut self, address: Address, index: StorageKey) -> StorageValue {
        self.journal_mut().tload(address, index)
    }

    /// Sets the transient storage value of `address` at `index`.
    fn tstore(&mut self, address: Address, index: StorageKey, value: StorageValue) {
        self.journal_mut().tstore(address, index, value)
    }

    /// Emits a log owned by `address` with given `LogData`.
    fn log(&mut self, log: Log) {
        self.journal_mut().log(log);
    }

    /// Marks `address` to be deleted, with funds transferred to `target`.
    fn selfdestruct(
        &mut self,
        address: Address,
        target: Address,
    ) -> Option<StateLoad<SelfDestructResult>> {
        self.journal_mut()
            .selfdestruct(address, target)
            .map_err(|e| {
                *self.error() = Err(e.into());
            })
            .ok()
    }
}
