//! This module contains [`Journal`] struct and implements [`JournalTr`] trait for it.
//!
//! Entry submodule contains [`JournalEntry`] and [`JournalEntryTr`] traits.
//! and inner submodule contains [`JournalInner`] struct that contains state.
pub mod entry;
pub mod inner;

pub use entry::{JournalEntry, JournalEntryTr};
pub use inner::JournalInner;

use bytecode::Bytecode;
use context_interface::{
    context::{SStoreResult, SelfDestructResult, StateLoad},
    journaled_state::{AccountLoad, JournalCheckpoint, JournalTr, TransferError},
};
use core::ops::{Deref, DerefMut};
use database_interface::Database;
use primitives::{hardfork::SpecId, Address, HashSet, Log, StorageKey, StorageValue, B256, U256};
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

/// Output of the journal after finalizing.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JournalOutput {
    /// Changes or touched accounts that loads, created or changed in the journal.
    pub state: EvmState,
    /// Logs that were emitted by contract calls.
    pub logs: Vec<Log>,
}

impl<DB: Database, ENTRY: JournalEntryTr> JournalTr for Journal<DB, ENTRY> {
    type Database = DB;
    type FinalOutput = JournalOutput;

    fn new(database: DB) -> Journal<DB, ENTRY> {
        Self {
            inner: JournalInner::new(),
            database,
        }
    }

    fn db_ref(&self) -> &Self::Database {
        &self.database
    }

    fn db(&mut self) -> &mut Self::Database {
        &mut self.database
    }

    fn sload(
        &mut self,
        address: Address,
        key: StorageKey,
    ) -> Result<StateLoad<StorageValue>, <Self::Database as Database>::Error> {
        self.inner.sload(&mut self.database, address, key)
    }

    fn sstore(
        &mut self,
        address: Address,
        key: StorageKey,
        value: StorageValue,
    ) -> Result<StateLoad<SStoreResult>, <Self::Database as Database>::Error> {
        self.inner.sstore(&mut self.database, address, key, value)
    }

    fn tload(&mut self, address: Address, key: StorageKey) -> StorageValue {
        self.inner.tload(address, key)
    }

    fn tstore(&mut self, address: Address, key: StorageKey, value: StorageValue) {
        self.inner.tstore(address, key, value)
    }

    fn log(&mut self, log: Log) {
        self.inner.log(log)
    }

    fn selfdestruct(
        &mut self,
        address: Address,
        target: Address,
    ) -> Result<StateLoad<SelfDestructResult>, DB::Error> {
        self.inner.selfdestruct(&mut self.database, address, target)
    }

    fn warm_account(&mut self, address: Address) {
        self.inner.warm_preloaded_addresses.insert(address);
    }

    fn warm_precompiles(&mut self, address: HashSet<Address>) {
        self.inner.precompiles = address;
        self.inner.warm_preloaded_addresses = self.inner.precompiles.clone();
    }

    #[inline]
    fn precompile_addresses(&self) -> &HashSet<Address> {
        &self.inner.precompiles
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
    ) -> Result<(), <Self::Database as Database>::Error> {
        self.inner
            .initial_account_load(&mut self.database, address, storage_keys)?;
        Ok(())
    }

    #[inline]
    fn set_spec_id(&mut self, spec_id: SpecId) {
        self.inner.spec = spec_id;
    }

    #[inline]
    fn transfer(
        &mut self,
        from: Address,
        to: Address,
        balance: U256,
    ) -> Result<Option<TransferError>, DB::Error> {
        self.inner.transfer(&mut self.database, from, to, balance)
    }

    #[inline]
    fn touch_account(&mut self, address: Address) {
        self.inner.touch(address);
    }

    #[inline]
    fn inc_account_nonce(&mut self, address: Address) -> Result<Option<u64>, DB::Error> {
        Ok(self.inner.inc_nonce(address))
    }

    #[inline]
    fn load_account(&mut self, address: Address) -> Result<StateLoad<&mut Account>, DB::Error> {
        self.inner.load_account(&mut self.database, address)
    }

    #[inline]
    fn load_account_code(
        &mut self,
        address: Address,
    ) -> Result<StateLoad<&mut Account>, DB::Error> {
        self.inner.load_code(&mut self.database, address)
    }

    #[inline]
    fn load_account_delegated(
        &mut self,
        address: Address,
    ) -> Result<StateLoad<AccountLoad>, DB::Error> {
        self.inner
            .load_account_delegated(&mut self.database, address)
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
    fn set_code_with_hash(&mut self, address: Address, code: Bytecode, hash: B256) {
        self.inner.set_code_with_hash(address, code, hash);
    }

    #[inline]
    fn clear(&mut self) {
        // Clears the inner journal state. Preserving only the spec and precompile addresses.
        let _ = self.inner.clear_and_take_output();
    }

    #[inline]
    fn create_account_checkpoint(
        &mut self,
        caller: Address,
        address: Address,
        balance: U256,
        spec_id: SpecId,
    ) -> Result<JournalCheckpoint, TransferError> {
        // Ignore error.
        self.inner
            .create_account_checkpoint(caller, address, balance, spec_id)
    }

    #[inline]
    fn finalize(&mut self) -> Self::FinalOutput {
        self.inner.clear_and_take_output()
    }
}
