pub mod performant_access;

use crate::{block::BlockEnv, cfg::CfgEnv, journaled_state::JournaledState, tx::TxEnv};
use bytecode::{
    eip7702::{EIP7702_MAGIC_BYTES, EIP7702_MAGIC_HASH},
    EOF_MAGIC_BYTES, EOF_MAGIC_HASH,
};
use context_interface::{
    block::BlockSetter, journaled_state::AccountLoad, transaction::TransactionSetter, Block,
    BlockGetter, Cfg, CfgGetter, DatabaseGetter, ErrorGetter, Journal, JournalGetter, Transaction,
    TransactionGetter,
};
use database_interface::{Database, EmptyDB};
use derive_where::derive_where;
use interpreter::{Host, SStoreResult, SelfDestructResult, StateLoad};
use primitives::{Address, Bytes, Log, B256, BLOCK_HASH_HISTORY, U256};
use specification::hardfork::SpecId;

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

impl Default for Context {
    fn default() -> Self {
        Self::new(EmptyDB::new(), SpecId::LATEST)
    }
}

impl Context {
    pub fn builder() -> Self {
        Self::new(EmptyDB::new(), SpecId::LATEST)
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

    /// Returns account code bytes and if address is cold loaded.
    ///
    /// In case of EOF account it will return `EOF_MAGIC` (0xEF00) as code.
    ///
    // TODO : Move this in Journaled state
    #[inline]
    pub fn code(&mut self, address: Address) -> Result<StateLoad<Bytes>, <DB as Database>::Error> {
        let a = self.journaled_state.load_account_code(address)?;
        // SAFETY: Safe to unwrap as load_code will insert code if it is empty.
        let code = a.info.code.as_ref().unwrap();

        let code = if code.is_eof() {
            EOF_MAGIC_BYTES.clone()
        } else if code.is_eip7702() {
            EIP7702_MAGIC_BYTES.clone()
        } else {
            code.original_bytes()
        };

        Ok(StateLoad::new(code, a.is_cold))
    }

    /// Gets code hash of address.
    ///
    /// In case of EOF account it will return `EOF_MAGIC_HASH`
    /// (the hash of `0xEF00`).
    ///
    // TODO : Move this in Journaled state
    #[inline]
    pub fn code_hash(
        &mut self,
        address: Address,
    ) -> Result<StateLoad<B256>, <DB as Database>::Error> {
        let acc = self.journaled_state.load_account_code(address)?;
        if acc.is_empty() {
            return Ok(StateLoad::new(B256::ZERO, acc.is_cold));
        }
        // SAFETY: Safe to unwrap as load_code will insert code if it is empty.
        let code = acc.info.code.as_ref().unwrap();

        let hash = if code.is_eof() {
            EOF_MAGIC_HASH
        } else if code.is_eip7702() {
            EIP7702_MAGIC_HASH
        } else {
            acc.info.code_hash
        };

        Ok(StateLoad::new(hash, acc.is_cold))
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
    fn block_hash(&mut self, requested_number: u64) -> Option<B256> {
        let block_number = self.block().number();

        let Some(diff) = block_number.checked_sub(requested_number) else {
            return Some(B256::ZERO);
        };

        // blockhash should push zero if number is same as current block number.
        if diff == 0 {
            return Some(B256::ZERO);
        }

        if diff <= BLOCK_HASH_HISTORY {
            return self
                .journaled_state
                .db()
                .block_hash(requested_number)
                .map_err(|e| self.error = Err(e))
                .ok();
        }

        Some(B256::ZERO)
    }

    fn load_account_delegated(&mut self, address: Address) -> Option<StateLoad<AccountLoad>> {
        self.journaled_state
            .load_account_delegated(address)
            .map_err(|e| self.error = Err(e))
            .ok()
    }

    fn balance(&mut self, address: Address) -> Option<StateLoad<U256>> {
        self.journaled_state
            .load_account(address)
            .map_err(|e| self.error = Err(e))
            .map(|acc| acc.map(|a| a.info.balance))
            .ok()
    }

    fn code(&mut self, address: Address) -> Option<StateLoad<Bytes>> {
        self.code(address).map_err(|e| self.error = Err(e)).ok()
    }

    fn code_hash(&mut self, address: Address) -> Option<StateLoad<B256>> {
        self.code_hash(address)
            .map_err(|e| self.error = Err(e))
            .ok()
    }

    fn sload(&mut self, address: Address, index: U256) -> Option<StateLoad<U256>> {
        self.journaled_state
            .sload(address, index)
            .map_err(|e| self.error = Err(e))
            .ok()
    }

    fn sstore(
        &mut self,
        address: Address,
        index: U256,
        value: U256,
    ) -> Option<StateLoad<SStoreResult>> {
        self.journaled_state
            .sstore(address, index, value)
            .map_err(|e| self.error = Err(e))
            .ok()
    }

    fn tload(&mut self, address: Address, index: U256) -> U256 {
        self.journaled_state.tload(address, index)
    }

    fn tstore(&mut self, address: Address, index: U256, value: U256) {
        self.journaled_state.tstore(address, index, value)
    }

    fn log(&mut self, log: Log) {
        self.journaled_state.log(log);
    }

    fn selfdestruct(
        &mut self,
        address: Address,
        target: Address,
    ) -> Option<StateLoad<SelfDestructResult>> {
        self.journaled_state
            .selfdestruct(address, target)
            .map_err(|e| self.error = Err(e))
            .ok()
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
