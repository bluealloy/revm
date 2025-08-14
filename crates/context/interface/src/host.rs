//! Host interface for external blockchain state access.

use crate::{
    context::{SStoreResult, SelfDestructResult, StateLoad},
    journaled_state::AccountLoad,
};
use auto_impl::auto_impl;
use primitives::{Address, Bytes, Log, StorageKey, StorageValue, B256, U256};

/// Host trait with all methods that are needed by the Interpreter.
///
/// This trait is implemented for all types that have `ContextTr` trait.
///
/// There are few groups of functions which are Block, Transaction, Config, Database and Journal functions.
#[auto_impl(&mut, Box)]
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
    fn block_number(&self) -> U256;
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

    /* Config */

    /// Max initcode size, calls `ContextTr::cfg().max_code_size().saturating_mul(2)`
    fn max_initcode_size(&self) -> usize;

    /* Database */

    /// Block hash, calls `ContextTr::journal_mut().db().block_hash(number)`
    fn block_hash(&mut self, number: u64) -> Option<B256>;

    /* Journal */

    /// Selfdestruct account, calls `ContextTr::journal_mut().selfdestruct(address, target)`
    fn selfdestruct(
        &mut self,
        address: Address,
        target: Address,
    ) -> Option<StateLoad<SelfDestructResult>>;

    /// Log, calls `ContextTr::journal_mut().log(log)`
    fn log(&mut self, log: Log);
    /// Sstore, calls `ContextTr::journal_mut().sstore(address, key, value)`
    fn sstore(
        &mut self,
        address: Address,
        key: StorageKey,
        value: StorageValue,
    ) -> Option<StateLoad<SStoreResult>>;

    /// Sload, calls `ContextTr::journal_mut().sload(address, key)`
    fn sload(&mut self, address: Address, key: StorageKey) -> Option<StateLoad<StorageValue>>;
    /// Tstore, calls `ContextTr::journal_mut().tstore(address, key, value)`
    fn tstore(&mut self, address: Address, key: StorageKey, value: StorageValue);
    /// Tload, calls `ContextTr::journal_mut().tload(address, key)`
    fn tload(&mut self, address: Address, key: StorageKey) -> StorageValue;
    /// Balance, calls `ContextTr::journal_mut().load_account(address)`
    fn balance(&mut self, address: Address) -> Option<StateLoad<U256>>;
    /// Load account delegated, calls `ContextTr::journal_mut().load_account_delegated(address)`
    fn load_account_delegated(&mut self, address: Address) -> Option<StateLoad<AccountLoad>>;
    /// Load account code, calls `ContextTr::journal_mut().load_account_code(address)`
    fn load_account_code(&mut self, address: Address) -> Option<StateLoad<Bytes>>;
    /// Load account code hash, calls `ContextTr::journal_mut().code_hash(address)`
    fn load_account_code_hash(&mut self, address: Address) -> Option<StateLoad<B256>>;
}

/// Dummy host that implements [`Host`] trait and  returns all default values.
#[derive(Debug)]
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

    fn block_number(&self) -> U256 {
        U256::ZERO
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
