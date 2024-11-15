use crate::{
    default::{block::BlockEnv, tx::TxEnv},
    getters::*,
    journaled_state::JournaledState,
};
use bytecode::{Bytecode, EOF_MAGIC_BYTES, EOF_MAGIC_HASH};
use database_interface::{Database, EmptyDB};
use derive_where::derive_where;
use interpreter::{as_u64_saturated, Host, SStoreResult, SelfDestructResult, StateLoad};
use primitives::{Address, Bytes, HashSet, Log, B256, BLOCK_HASH_HISTORY, U256};
use specification::hardfork::SpecId;

use wiring::{
    journaled_state::{AccountLoad, Eip7702CodeLoad},
    result::EVMError,
    Block, CfgEnv, Transaction,
};

/// EVM context contains data that EVM needs for execution.
#[derive_where(Clone, Debug; BLOCK, SPEC, CHAIN, TX, DB, <DB as Database>::Error)]
pub struct Context<BLOCK = BlockEnv, TX = TxEnv, SPEC = SpecId, DB: Database = EmptyDB, CHAIN = ()>
{
    /// Transaction information.
    pub tx: TX,
    /// Block information.
    pub block: BLOCK,
    /// Configurations.
    pub cfg: CfgEnv,
    /// EVM State with journaling support and database.
    pub journaled_state: JournaledState<DB>,
    /// Inner context.
    pub chain: CHAIN,
    /// TODO include it inside CfgEnv.
    pub spec: SPEC,
    /// Error that happened during execution.
    pub error: Result<(), <DB as Database>::Error>,
}

impl<BLOCK: Block + Default, TX: Transaction + Default, SPEC, DB: Database, CHAIN: Default>
    Context<BLOCK, TX, SPEC, DB, CHAIN>
{
    pub fn new(db: DB, spec: SPEC) -> Self {
        Self {
            tx: TX::default(),
            block: BLOCK::default(),
            cfg: CfgEnv::default(),
            journaled_state: JournaledState::new(SpecId::LATEST, db, HashSet::default()),
            spec,
            chain: Default::default(),
            error: Ok(()),
        }
    }
}
impl<BLOCK: Block, TX: Transaction, SPEC, DB: Database, CHAIN> Context<BLOCK, TX, SPEC, DB, CHAIN> {
    /// Return account code bytes and if address is cold loaded.
    ///
    /// In case of EOF account it will return `EOF_MAGIC` (0xEF00) as code.
    ///
    /// TODO move this in Journaled state
    #[inline]
    pub fn code(
        &mut self,
        address: Address,
    ) -> Result<Eip7702CodeLoad<Bytes>, <DB as Database>::Error> {
        let a = self.journaled_state.load_code(address)?;
        // SAFETY: safe to unwrap as load_code will insert code if it is empty.
        let code = a.info.code.as_ref().unwrap();
        if code.is_eof() {
            return Ok(Eip7702CodeLoad::new_not_delegated(
                EOF_MAGIC_BYTES.clone(),
                a.is_cold,
            ));
        }

        if let Bytecode::Eip7702(code) = code {
            let address = code.address();
            let is_cold = a.is_cold;

            let delegated_account = self.journaled_state.load_code(address)?;

            // SAFETY: safe to unwrap as load_code will insert code if it is empty.
            let delegated_code = delegated_account.info.code.as_ref().unwrap();

            let bytes = if delegated_code.is_eof() {
                EOF_MAGIC_BYTES.clone()
            } else {
                delegated_code.original_bytes()
            };

            return Ok(Eip7702CodeLoad::new(
                StateLoad::new(bytes, is_cold),
                delegated_account.is_cold,
            ));
        }

        Ok(Eip7702CodeLoad::new_not_delegated(
            code.original_bytes(),
            a.is_cold,
        ))
    }

    /// Get code hash of address.
    ///
    /// In case of EOF account it will return `EOF_MAGIC_HASH`
    /// (the hash of `0xEF00`).
    ///
    /// TODO move this in Journaled state
    #[inline]
    pub fn code_hash(
        &mut self,
        address: Address,
    ) -> Result<Eip7702CodeLoad<B256>, <DB as Database>::Error> {
        let acc = self.journaled_state.load_code(address)?;
        if acc.is_empty() {
            return Ok(Eip7702CodeLoad::new_not_delegated(B256::ZERO, acc.is_cold));
        }
        // SAFETY: safe to unwrap as load_code will insert code if it is empty.
        let code = acc.info.code.as_ref().unwrap();

        // If bytecode is EIP-7702 then we need to load the delegated account.
        if let Bytecode::Eip7702(code) = code {
            let address = code.address();
            let is_cold = acc.is_cold;

            let delegated_account = self.journaled_state.load_code(address)?;

            let hash = if delegated_account.is_empty() {
                B256::ZERO
            } else if delegated_account.info.code.as_ref().unwrap().is_eof() {
                EOF_MAGIC_HASH
            } else {
                delegated_account.info.code_hash
            };

            return Ok(Eip7702CodeLoad::new(
                StateLoad::new(hash, is_cold),
                delegated_account.is_cold,
            ));
        }

        let hash = if code.is_eof() {
            EOF_MAGIC_HASH
        } else {
            acc.info.code_hash
        };

        Ok(Eip7702CodeLoad::new_not_delegated(hash, acc.is_cold))
    }
}

impl<BLOCK: Block, TX: Transaction, SPEC, DB: Database, CHAIN> Host
    for Context<BLOCK, TX, SPEC, DB, CHAIN>
{
    type BLOCK = BLOCK;
    type TX = TX;

    fn tx(&self) -> &Self::TX {
        &self.tx
    }

    fn block(&self) -> &Self::BLOCK {
        &self.block
    }

    fn cfg(&self) -> &CfgEnv {
        &self.cfg
    }

    fn block_hash(&mut self, requested_number: u64) -> Option<B256> {
        let block_number = as_u64_saturated!(*self.block().number());

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
                .database
                .block_hash(requested_number)
                .map_err(|e| self.error = Err(e))
                .ok();
        }

        Some(B256::ZERO)
    }

    fn load_account_delegated(&mut self, address: Address) -> Option<AccountLoad> {
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

    fn code(&mut self, address: Address) -> Option<Eip7702CodeLoad<Bytes>> {
        self.code(address).map_err(|e| self.error = Err(e)).ok()
    }

    fn code_hash(&mut self, address: Address) -> Option<Eip7702CodeLoad<B256>> {
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

impl<BLOCK, TX, DB: Database, SPEC, CHAIN> CfgGetter for Context<BLOCK, TX, SPEC, DB, CHAIN> {
    type Cfg = CfgEnv;

    fn cfg(&self) -> &Self::Cfg {
        &self.cfg
    }
}

impl<BLOCK, TX, SPEC, DB: Database, CHAIN> JournalStateGetter
    for Context<BLOCK, TX, SPEC, DB, CHAIN>
{
    type Journal = JournaledState<DB>;

    fn journal(&mut self) -> &mut Self::Journal {
        &mut self.journaled_state
    }
}

impl<BLOCK, TX, SPEC, DB: Database, CHAIN> DatabaseGetter for Context<BLOCK, TX, SPEC, DB, CHAIN> {
    type Database = DB;

    fn db(&mut self) -> &mut Self::Database {
        &mut self.journaled_state.database
    }
}

impl<BLOCK, TX: Transaction, SPEC, DB: Database, CHAIN> ErrorGetter
    for Context<BLOCK, TX, SPEC, DB, CHAIN>
{
    type Error = EVMError<DB::Error, TX::TransactionError>;

    fn take_error(&mut self) -> Result<(), Self::Error> {
        core::mem::replace(&mut self.error, Ok(())).map_err(EVMError::Database)
    }
}

impl<BLOCK, TX: Transaction, SPEC, DB: Database, CHAIN> TransactionGetter
    for Context<BLOCK, TX, SPEC, DB, CHAIN>
{
    type Transaction = TX;

    fn tx(&self) -> &Self::Transaction {
        &self.tx
    }
}
impl<BLOCK: Block, TX, SPEC, DB: Database, CHAIN> BlockGetter
    for Context<BLOCK, TX, SPEC, DB, CHAIN>
{
    type Block = BLOCK;

    fn block(&self) -> &Self::Block {
        &self.block
    }
}

impl<BLOCK: Block, TX: Transaction, SPEC, DB: Database, CHAIN> AllGetters
    for Context<BLOCK, TX, SPEC, DB, CHAIN>
{
}
