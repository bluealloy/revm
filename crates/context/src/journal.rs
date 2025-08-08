//! This module contains [`Journal`] struct and implements [`JournalTr`] trait for it.
//!
//! Entry submodule contains [`JournalEntry`] and [`JournalEntryTr`] traits.
//! and inner submodule contains [`JournalInner`] struct that contains state.
pub mod entry;
pub mod inner;
pub mod warm_addresses;

pub use entry::{JournalEntry, JournalEntryTr};
pub use inner::JournalInner;

use bytecode::Bytecode;
use context_interface::{
    context::{SStoreResult, SelfDestructResult, StateLoad},
    journaled_state::{AccountLoad, JournalCheckpoint, JournalTr, TransferError},
};
use core::ops::{Deref, DerefMut};
use database_interface::Database;
use primitives::{
    hardfork::SpecId, Address, AddressAndId, AddressOrId, HashSet, Log, StorageKey, StorageValue,
    B256, U256,
};
use state::{Account, EvmState};
use std::vec::Vec;

/// A journal of state changes internal to the EVM
///
/// On each additional call, the depth of the journaled state is increased (`depth`) and a new journal is added.
///
/// The journal contains every state change that happens within that call, making it possible to revert changes made in a specific call.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Journal<DB, ENTRY = JournalEntry>
where
    ENTRY: JournalEntryTr,
{
    /// Database
    pub database: DB,
    /// Inner journal state.
    pub inner: JournalInner<ENTRY>,
}

impl<DB, ENTRY> Deref for Journal<DB, ENTRY>
where
    ENTRY: JournalEntryTr,
{
    type Target = JournalInner<ENTRY>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<DB, ENTRY> DerefMut for Journal<DB, ENTRY>
where
    ENTRY: JournalEntryTr,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<DB, ENTRY: JournalEntryTr> Journal<DB, ENTRY> {
    /// Creates a new JournaledState by copying state data from a JournalInit and provided database.
    /// This allows reusing the state, logs, and other data from a previous execution context while
    /// connecting it to a different database backend.
    pub fn new_with_inner(database: DB, inner: JournalInner<ENTRY>) -> Self {
        Self { database, inner }
    }

    /// Consumes the [`Journal`] and returns [`JournalInner`].
    ///
    /// If you need to preserve the original journal, use [`Self::to_inner`] instead which clones the state.
    pub fn into_init(self) -> JournalInner<ENTRY> {
        self.inner
    }
}

impl<DB, ENTRY: JournalEntryTr + Clone> Journal<DB, ENTRY> {
    /// Creates a new [`JournalInner`] by cloning all internal state data (state, storage, logs, etc)
    /// This allows creating a new journaled state with the same state data but without
    /// carrying over the original database.
    ///
    /// This is useful when you want to reuse the current state for a new transaction or
    /// execution context, but want to start with a fresh database.
    pub fn to_inner(&self) -> JournalInner<ENTRY> {
        self.inner.clone()
    }
}

impl<DB: Database, ENTRY: JournalEntryTr> JournalTr for Journal<DB, ENTRY> {
    type Database = DB;
    type State = EvmState;

    fn new(database: DB) -> Journal<DB, ENTRY> {
        Self {
            inner: JournalInner::new(),
            database,
        }
    }

    fn db(&self) -> &Self::Database {
        &self.database
    }

    fn db_mut(&mut self) -> &mut Self::Database {
        &mut self.database
    }

    fn sload(
        &mut self,
        address_or_id: AddressOrId,
        key: StorageKey,
    ) -> Result<StateLoad<StorageValue>, <Self::Database as Database>::Error> {
        self.inner.sload(&mut self.database, address_or_id, key)
    }

    fn sstore(
        &mut self,
        address_or_id: AddressOrId,
        key: StorageKey,
        value: StorageValue,
    ) -> Result<StateLoad<SStoreResult>, <Self::Database as Database>::Error> {
        self.inner
            .sstore(&mut self.database, address_or_id, key, value)
    }

    fn tload(&mut self, address_or_id: AddressOrId, key: StorageKey) -> StorageValue {
        self.inner.tload(address_or_id, key)
    }

    fn tstore(&mut self, address_or_id: AddressOrId, key: StorageKey, value: StorageValue) {
        self.inner.tstore(address_or_id, key, value)
    }

    fn log(&mut self, log: Log) {
        self.inner.log(log)
    }

    fn selfdestruct(
        &mut self,
        address_or_id: AddressOrId,
        target: Address,
    ) -> Result<StateLoad<SelfDestructResult>, DB::Error> {
        self.inner
            .selfdestruct(&mut self.database, address_or_id, target)
    }

    fn warm_coinbase_account(&mut self, address: Address) {
        self.inner.warm_addresses.set_coinbase(address);
    }

    fn warm_precompiles(&mut self, precompiles: HashSet<Address>) {
        self.inner
            .warm_addresses
            .set_precompile_addresses(precompiles);
    }

    #[inline]
    fn precompile_addresses(&self) -> &HashSet<Address> {
        self.inner.warm_addresses.precompiles()
    }

    /// Returns call depth.
    #[inline]
    fn depth(&self) -> usize {
        self.inner.depth
    }

    #[inline]
    fn warm_account_and_storage(
        &mut self,
        address: Address,
        storage_keys: impl IntoIterator<Item = StorageKey>,
    ) -> Result<AddressAndId, <Self::Database as Database>::Error> {
        self.inner
            .load_account_optional(
                &mut self.database,
                AddressOrId::Address(address),
                false,
                storage_keys,
            )
            .map(|i| i.1)
    }

    #[inline]
    fn set_spec_id(&mut self, spec_id: SpecId) {
        self.inner.spec = spec_id;
    }

    #[inline]
    fn transfer(
        &mut self,
        from: AddressOrId,
        to: AddressOrId,
        balance: U256,
    ) -> Result<Option<TransferError>, DB::Error> {
        self.inner.transfer(&mut self.database, from, to, balance)
    }

    #[inline]
    fn touch_account(&mut self, address_or_id: AddressOrId) {
        self.inner.touch(&address_or_id);
    }

    #[inline]
    fn caller_accounting_journal_entry(
        &mut self,
        address_or_id: AddressOrId,
        old_balance: U256,
        bump_nonce: bool,
    ) {
        self.inner
            .caller_accounting_journal_entry(address_or_id, old_balance, bump_nonce);
    }

    /// Increments the balance of the account.
    #[inline]
    fn balance_incr(
        &mut self,
        address_or_id: AddressOrId,
        balance: U256,
    ) -> Result<AddressAndId, <Self::Database as Database>::Error> {
        self.inner
            .balance_incr(&mut self.database, address_or_id, balance)
    }

    /// Increments the nonce of the account.
    #[inline]
    fn nonce_bump_journal_entry(&mut self, address_or_id: AddressOrId) {
        self.inner.nonce_bump_journal_entry(address_or_id)
    }

    #[inline]
    fn load_account(
        &mut self,
        address_or_id: AddressOrId,
    ) -> Result<StateLoad<(&mut Account, AddressAndId)>, DB::Error> {
        self.inner.load_account(&mut self.database, address_or_id)
    }

    #[inline]
    fn load_account_code(
        &mut self,
        address_or_id: AddressOrId,
    ) -> Result<StateLoad<(&mut Account, AddressAndId)>, DB::Error> {
        self.inner.load_code(&mut self.database, address_or_id)
    }

    #[inline]
    fn load_account_delegated(
        &mut self,
        address_or_id: AddressOrId,
    ) -> Result<StateLoad<AccountLoad>, DB::Error> {
        self.inner
            .load_account_delegated(&mut self.database, address_or_id)
    }

    #[inline]
    fn checkpoint(&mut self) -> JournalCheckpoint {
        self.inner.checkpoint()
    }

    #[inline]
    fn checkpoint_commit(&mut self) {
        self.inner.checkpoint_commit()
    }

    #[inline]
    fn checkpoint_revert(&mut self, checkpoint: JournalCheckpoint) {
        self.inner.checkpoint_revert(checkpoint)
    }

    #[inline]
    fn set_code_with_hash(&mut self, address_or_id: AddressOrId, code: Bytecode, hash: B256) {
        self.inner.set_code_with_hash(address_or_id, code, hash);
    }

    #[inline]
    fn create_account_checkpoint(
        &mut self,
        caller_or_id: AddressOrId,
        target_or_id: AddressOrId,
        balance: U256,
        spec_id: SpecId,
    ) -> Result<JournalCheckpoint, TransferError> {
        // Ignore error.
        self.inner
            .create_account_checkpoint(caller_or_id, target_or_id, balance, spec_id)
    }

    #[inline]
    fn take_logs(&mut self) -> Vec<Log> {
        self.inner.take_logs()
    }

    #[inline]
    fn commit_tx(&mut self) {
        self.inner.commit_tx()
    }

    #[inline]
    fn discard_tx(&mut self) {
        self.inner.discard_tx();
    }

    /// Clear current journal resetting it to initial state and return changes state.
    #[inline]
    fn finalize(&mut self) -> Self::State {
        self.inner.finalize()
    }
}
