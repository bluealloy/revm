use context_interface::{
    context::{ContextTr, SStoreResult, SelfDestructResult, StateLoad},
    journaled_state::AccountLoad,
    Block, Cfg, Database, JournalTr, LocalContextTr, Transaction, TransactionType,
};
use primitives::{Address, Bytes, Log, StorageKey, StorageValue, B256, U256};

use crate::instructions::utility::IntoU256;

/// Host trait with all methods that are needed by the Interpreter.
///
/// This trait is implemented for all types that have `ContextTr` trait.
///
/// There are few groups of functions which are Block, Transaction, Config, Database and Journal functions.
pub trait Host {
    /* Block */

    /// Block basefee, calls ContextTr::block().basefee()
    fn basefee(&self) -> U256;
    /// Block blob gasprice, calls `ContextTr::block().blob_gasprice()`
    fn blob_gasprice(&self) -> U256;
    /// Block gas limit, calls ContextTr::block().gas_limit()
    fn gas_limit(&self) -> U256;
    /// Block difficulty, calls ContextTr::block().difficulty()
    fn difficulty(&self) -> U256;
    /// Block prevrandao, calls ContextTr::block().prevrandao()
    fn prevrandao(&self) -> Option<U256>;
    /// Block number, calls ContextTr::block().number()
    fn block_number(&self) -> u64;
    /// Block timestamp, calls ContextTr::block().timestamp()
    fn timestamp(&self) -> U256;
    /// Block beneficiary, calls ContextTr::block().beneficiary()
    fn beneficiary(&self) -> Address;
    /// Chain id, calls ContextTr::cfg().chain_id()
    fn chain_id(&self) -> U256;

    /* Transaction */

    /// Transaction effective gas price, calls `ContextTr::tx().effective_gas_price(basefee as u128)`
    fn effective_gas_price(&self) -> U256;
    /// Transaction caller, calls `ContextTr::tx().caller()`
    fn caller(&self) -> Address;
    /// Transaction blob hash, calls `ContextTr::tx().blob_hash(number)`
    fn blob_hash(&self, number: usize) -> Option<U256>;
    /// Initcodes mapped to the hash.
    fn initcode_by_hash(&mut self, hash: B256) -> Option<Bytes>;

    /* Config */

    /// Max initcode size, calls `ContextTr::cfg().max_code_size().saturating_mul(2)`
    fn max_initcode_size(&self) -> usize;

    /* Database */

    /// Block hash, calls `ContextTr::journal().db().block_hash(number)`
    fn block_hash(&mut self, number: u64) -> Option<B256>;

    /* Journal */

    /// Selfdestruct account, calls `ContextTr::journal().selfdestruct(address, target)`
    fn selfdestruct(
        &mut self,
        address: Address,
        target: Address,
    ) -> Option<StateLoad<SelfDestructResult>>;

    /// Log, calls `ContextTr::journal().log(log)`
    fn log(&mut self, log: Log);
    /// Sstore, calls `ContextTr::journal().sstore(address, key, value)`
    fn sstore(
        &mut self,
        address: Address,
        key: StorageKey,
        value: StorageValue,
    ) -> Option<StateLoad<SStoreResult>>;

    /// Sload, calls `ContextTr::journal().sload(address, key)`
    fn sload(&mut self, address: Address, key: StorageKey) -> Option<StateLoad<StorageValue>>;
    /// Tstore, calls `ContextTr::journal().tstore(address, key, value)`
    fn tstore(&mut self, address: Address, key: StorageKey, value: StorageValue);
    /// Tload, calls `ContextTr::journal().tload(address, key)`
    fn tload(&mut self, address: Address, key: StorageKey) -> StorageValue;
    /// Balance, calls `ContextTr::journal().load_account(address)`
    fn balance(&mut self, address: Address) -> Option<StateLoad<U256>>;
    /// Load account delegated, calls `ContextTr::journal().load_account_delegated(address)`
    fn load_account_delegated(&mut self, address: Address) -> Option<StateLoad<AccountLoad>>;
    /// Load account code, calls `ContextTr::journal().load_account_code(address)`
    fn load_account_code(&mut self, address: Address) -> Option<StateLoad<Bytes>>;
    /// Load account code hash, calls `ContextTr::journal().code_hash(address)`
    fn load_account_code_hash(&mut self, address: Address) -> Option<StateLoad<B256>>;
}

impl<CTX: ContextTr> Host for CTX {
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
        self.block().prevrandao().map(|r| r.into_u256())
    }

    fn block_number(&self) -> u64 {
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
        let tx = &self.tx();
        if tx.tx_type() != TransactionType::Eip4844 {
            return None;
        }
        tx.blob_versioned_hashes()
            .get(number)
            .map(|t| U256::from_be_bytes(t.0))
    }

    fn initcode_by_hash(&mut self, hash: B256) -> Option<Bytes> {
        self.local().get_validated_initcode(hash)
    }

    /* Config */

    fn max_initcode_size(&self) -> usize {
        self.cfg().max_code_size().saturating_mul(2)
    }

    /* Database */

    fn block_hash(&mut self, requested_number: u64) -> Option<B256> {
        self.journal()
            .db()
            .block_hash(requested_number)
            .map_err(|e| {
                *self.error() = Err(e.into());
            })
            .ok()
    }

    /* Journal */

    fn load_account_delegated(&mut self, address: Address) -> Option<StateLoad<AccountLoad>> {
        self.journal()
            .load_account_delegated(address)
            .map_err(|e| {
                *self.error() = Err(e.into());
            })
            .ok()
    }

    /// Gets balance of `address` and if the account is cold.
    fn balance(&mut self, address: Address) -> Option<StateLoad<U256>> {
        self.journal()
            .load_account(address)
            .map(|acc| acc.map(|a| a.info.balance))
            .map_err(|e| {
                *self.error() = Err(e.into());
            })
            .ok()
    }

    /// Gets code of `address` and if the account is cold.
    fn load_account_code(&mut self, address: Address) -> Option<StateLoad<Bytes>> {
        self.journal()
            .code(address)
            .map_err(|e| {
                *self.error() = Err(e.into());
            })
            .ok()
    }

    /// Gets code hash of `address` and if the account is cold.
    fn load_account_code_hash(&mut self, address: Address) -> Option<StateLoad<B256>> {
        self.journal()
            .code_hash(address)
            .map_err(|e| {
                *self.error() = Err(e.into());
            })
            .ok()
    }

    /// Gets storage value of `address` at `index` and if the account is cold.
    fn sload(&mut self, address: Address, index: StorageKey) -> Option<StateLoad<StorageValue>> {
        self.journal()
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
        self.journal()
            .sstore(address, index, value)
            .map_err(|e| {
                *self.error() = Err(e.into());
            })
            .ok()
    }

    /// Gets the transient storage value of `address` at `index`.
    fn tload(&mut self, address: Address, index: StorageKey) -> StorageValue {
        self.journal().tload(address, index)
    }

    /// Sets the transient storage value of `address` at `index`.
    fn tstore(&mut self, address: Address, index: StorageKey, value: StorageValue) {
        self.journal().tstore(address, index, value)
    }

    /// Emits a log owned by `address` with given `LogData`.
    fn log(&mut self, log: Log) {
        self.journal().log(log);
    }

    /// Marks `address` to be deleted, with funds transferred to `target`.
    fn selfdestruct(
        &mut self,
        address: Address,
        target: Address,
    ) -> Option<StateLoad<SelfDestructResult>> {
        self.journal()
            .selfdestruct(address, target)
            .map_err(|e| {
                *self.error() = Err(e.into());
            })
            .ok()
    }
}

/// Dummy host that implements [`Host`] trait and  returns all default values.
pub struct DummyHost;

impl Host for DummyHost {
    fn basefee(&self) -> U256 {
        U256::ZERO
    }

    fn blob_gasprice(&self) -> U256 {
        U256::ZERO
    }

    fn gas_limit(&self) -> U256 {
        U256::ZERO
    }

    fn difficulty(&self) -> U256 {
        U256::ZERO
    }

    fn prevrandao(&self) -> Option<U256> {
        None
    }

    fn block_number(&self) -> u64 {
        0
    }

    fn timestamp(&self) -> U256 {
        U256::ZERO
    }

    fn beneficiary(&self) -> Address {
        Address::ZERO
    }

    fn chain_id(&self) -> U256 {
        U256::ZERO
    }

    fn effective_gas_price(&self) -> U256 {
        U256::ZERO
    }

    fn caller(&self) -> Address {
        Address::ZERO
    }

    fn initcode_by_hash(&mut self, _hash: B256) -> Option<Bytes> {
        None
    }

    fn blob_hash(&self, _number: usize) -> Option<U256> {
        None
    }

    fn max_initcode_size(&self) -> usize {
        0
    }

    fn block_hash(&mut self, _number: u64) -> Option<B256> {
        None
    }

    fn selfdestruct(
        &mut self,
        _address: Address,
        _target: Address,
    ) -> Option<StateLoad<SelfDestructResult>> {
        None
    }

    fn log(&mut self, _log: Log) {}

    fn sstore(
        &mut self,
        _address: Address,
        _key: StorageKey,
        _value: StorageValue,
    ) -> Option<StateLoad<SStoreResult>> {
        None
    }

    fn sload(&mut self, _address: Address, _key: StorageKey) -> Option<StateLoad<StorageValue>> {
        None
    }

    fn tstore(&mut self, _address: Address, _key: StorageKey, _value: StorageValue) {}

    fn tload(&mut self, _address: Address, _key: StorageKey) -> StorageValue {
        StorageValue::ZERO
    }

    fn balance(&mut self, _address: Address) -> Option<StateLoad<U256>> {
        None
    }

    fn load_account_delegated(&mut self, _address: Address) -> Option<StateLoad<AccountLoad>> {
        None
    }

    fn load_account_code(&mut self, _address: Address) -> Option<StateLoad<Bytes>> {
        None
    }

    fn load_account_code_hash(&mut self, _address: Address) -> Option<StateLoad<B256>> {
        None
    }
}
