//! Journaled state trait [`JournalTr`] and related types.
use crate::context::{SStoreResult, SelfDestructResult};
use core::ops::{Deref, DerefMut};
use database_interface::Database;
use primitives::{
    hardfork::SpecId, AccountId, Address, AddressAndId, AddressOrId, Bytes, HashSet, Log,
    StorageKey, StorageValue, B256, U256,
};
use state::{Account, Bytecode};
use std::vec::Vec;

/// Trait that contains database and journal of all changes that were made to the state.
pub trait JournalTr {
    /// Database type that is used in the journal.
    type Database: Database;
    /// State type that is returned by the journal after finalization.
    type State;

    /// Creates new Journaled state.
    ///
    /// Dont forget to set spec_id.
    fn new(database: Self::Database) -> Self;

    /// Returns a mutable reference to the database.
    fn db_mut(&mut self) -> &mut Self::Database;

    /// Returns an immutable reference to the database.
    fn db(&self) -> &Self::Database;

    /// Returns the storage value from Journal state.
    ///
    /// Loads the storage from database if not found in Journal state.
    fn sload(
        &mut self,
        address_or_id: AddressOrId,
        key: StorageKey,
    ) -> Result<StateLoad<StorageValue>, <Self::Database as Database>::Error>;

    /// Stores the storage value in Journal state.
    fn sstore(
        &mut self,
        address_or_id: AddressOrId,
        key: StorageKey,
        value: StorageValue,
    ) -> Result<StateLoad<SStoreResult>, <Self::Database as Database>::Error>;

    /// Loads transient storage value.
    fn tload(&mut self, address_or_id: AddressOrId, key: StorageKey) -> StorageValue;

    /// Stores transient storage value.
    fn tstore(&mut self, address_or_id: AddressOrId, key: StorageKey, value: StorageValue);

    /// Logs the log in Journal state.
    fn log(&mut self, log: Log);

    /// Marks the account for selfdestruction and transfers all the balance to the target.
    fn selfdestruct(
        &mut self,
        address_or_id: AddressOrId,
        target: Address,
    ) -> Result<StateLoad<SelfDestructResult>, <Self::Database as Database>::Error>;

    /// Warms the account and storage. Warming of account needs address.
    fn warm_account_and_storage(
        &mut self,
        address: Address,
        storage_keys: impl IntoIterator<Item = StorageKey>,
    ) -> Result<AddressAndId, <Self::Database as Database>::Error>;

    /// Warms the account. Internally calls [`JournalTr::warm_account_and_storage`] with empty storage keys.
    fn warm_account(
        &mut self,
        address: Address,
    ) -> Result<AddressAndId, <Self::Database as Database>::Error> {
        self.warm_account_and_storage(address, [])
    }

    /// Warms the precompiles. Precompile account will not be fetched from database
    ///
    /// Warming of precompiles assumes that account are frequently accessed and it will
    /// be considered warm even when loaded from database.F
    fn warm_precompiles(&mut self, addresses: HashSet<Address>);

    /// Returns the addresses of the precompiles.
    fn precompile_addresses(&self) -> &HashSet<Address>;

    /// Sets the spec id.
    fn set_spec_id(&mut self, spec_id: SpecId);

    /// Touches the account
    fn touch_account(&mut self, address_or_id: AddressOrId);

    /// Transfers the balance from one account to another.
    fn transfer(&mut self, from: AccountId, to: AccountId, balance: U256) -> Option<TransferError>;

    /// Increments the balance of the account.
    fn caller_accounting_journal_entry(
        &mut self,
        address_or_id: AddressOrId,
        old_balance: U256,
        bump_nonce: bool,
    );

    /// Increments the balance of the account.
    fn balance_incr(
        &mut self,
        address_or_id: AddressOrId,
        balance: U256,
    ) -> Result<AddressAndId, <Self::Database as Database>::Error>;

    /// Increments the balance of the account.
    fn balance_incr_by_id(&mut self, account_id: AccountId, balance: U256);

    /// Increments the nonce of the account.
    fn nonce_bump_journal_entry(&mut self, address_or_id: AddressOrId);

    /// Loads the account.
    fn load_account(
        &mut self,
        address_or_id: AddressOrId,
    ) -> Result<StateLoad<(&mut Account, AddressAndId)>, <Self::Database as Database>::Error>;

    /// Loads the account code.
    fn load_account_code(
        &mut self,
        address_or_id: AddressOrId,
    ) -> Result<StateLoad<(&mut Account, AddressAndId)>, <Self::Database as Database>::Error>;

    /// Returns the account by id.
    fn get_account_mut(&mut self, account_id: AccountId) -> (&mut Account, AddressAndId);

    /// Fast fetch the account by id.
    fn get_account(&mut self, account_id: AccountId) -> (&Account, AddressAndId);

    /// Sets the caller id.
    fn set_caller_address_id(&mut self, id: AddressAndId);

    /// Sets the coinbase id.
    fn set_coinbase_address_id(&mut self, id: AddressAndId);

    /// Returns the coinbase address and id.
    fn coinbase_address_id(&self) -> Option<AddressAndId>;

    /// Returns the caller address and id.
    fn caller_address_id(&self) -> Option<AddressAndId>;

    /// Sets the tx target address and id.
    fn set_tx_target_address_id(&mut self, id: AddressAndId, delegated: Option<AddressAndId>);

    /// Returns the tx target address and id.
    fn tx_target_address_id(&self) -> Option<(AddressAndId, Option<AddressAndId>)>;

    /// Loads the account delegated.
    fn load_account_delegated(
        &mut self,
        address_or_id: AddressOrId,
    ) -> Result<StateLoad<AccountLoad>, <Self::Database as Database>::Error>;

    /// Sets bytecode with hash. Assume that account is warm.
    fn set_code_with_hash(&mut self, address_or_id: AddressOrId, code: Bytecode, hash: B256);

    /// Sets bytecode and calculates hash.
    ///
    /// Assume account is warm.
    #[inline]
    fn set_code(&mut self, address_or_id: AddressOrId, code: Bytecode) {
        let hash = code.hash_slow();
        self.set_code_with_hash(address_or_id, code, hash);
    }

    /// Returns account code bytes and if address is cold loaded.
    #[inline]
    fn code(
        &mut self,
        address_or_id: AddressOrId,
    ) -> Result<(StateLoad<Bytes>, AddressAndId), <Self::Database as Database>::Error> {
        let a = self.load_account_code(address_or_id)?;
        // SAFETY: Safe to unwrap as load_code will insert code if it is empty.
        let code = a.0.info.code.as_ref().unwrap();
        let code = code.original_bytes();

        Ok((StateLoad::new(code, a.is_cold), a.1))
    }

    /// Gets code hash of account.
    fn code_hash(
        &mut self,
        address_or_id: AddressOrId,
    ) -> Result<(StateLoad<B256>, AddressAndId), <Self::Database as Database>::Error> {
        let acc = self.load_account(address_or_id)?;
        if acc.0.is_empty() {
            return Ok((StateLoad::new(B256::ZERO, acc.is_cold), acc.1));
        }
        let hash = acc.0.info.code_hash;
        Ok((StateLoad::new(hash, acc.is_cold), acc.1))
    }

    /// Called at the end of the transaction to clean all residue data from journal.
    fn clear(&mut self) {
        let _ = self.finalize();
    }

    /// Creates a checkpoint of the current state. State can be revert to this point
    /// if needed.
    fn checkpoint(&mut self) -> JournalCheckpoint;

    /// Commits the changes made since the last checkpoint.
    fn checkpoint_commit(&mut self);

    /// Reverts the changes made since the last checkpoint.
    fn checkpoint_revert(&mut self, checkpoint: JournalCheckpoint);

    /// Creates a checkpoint of the account creation.
    fn create_account_checkpoint(
        &mut self,
        caller_or_id: AddressOrId,
        address_or_id: AddressOrId,
        balance: U256,
        spec_id: SpecId,
    ) -> Result<JournalCheckpoint, TransferError>;

    /// Returns the depth of the journal.
    fn depth(&self) -> usize;

    /// Take logs from journal.
    fn take_logs(&mut self) -> Vec<Log>;

    /// Commit current transaction journal and returns transaction logs.
    fn commit_tx(&mut self);

    /// Discard current transaction journal by removing journal entries and logs and incrementing the transaction id.
    ///
    /// This function is useful to discard intermediate state that is interrupted by error and it will not revert
    /// any already committed changes and it is safe to call it multiple times.
    fn discard_tx(&mut self);

    /// Clear current journal resetting it to initial state and return changes state.
    fn finalize(&mut self) -> Self::State;
}

/// Transfer and creation result
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TransferError {
    /// Caller does not have enough funds
    OutOfFunds,
    /// Overflow in target account
    OverflowPayment,
    /// Create collision.
    CreateCollision,
}

/// SubRoutine checkpoint that will help us to go back from this
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JournalCheckpoint {
    /// Checkpoint to where on revert we will go back to.
    pub log_i: usize,
    /// Checkpoint to where on revert we will go back to and revert other journal entries.
    pub journal_i: usize,
}

/// State load information that contains the data and if the account or storage is cold loaded
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StateLoad<T> {
    /// Returned data
    pub data: T,
    /// Is account is cold loaded
    pub is_cold: bool,
}

impl<T> Deref for StateLoad<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> DerefMut for StateLoad<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl<T> StateLoad<T> {
    /// Returns a new [`StateLoad`] with the given data and cold load status.
    pub fn new(data: T, is_cold: bool) -> Self {
        Self { data, is_cold }
    }

    /// Maps the data of the [`StateLoad`] to a new value.
    ///
    /// Useful for transforming the data of the [`StateLoad`] without changing the cold load status.
    pub fn map<B, F>(self, f: F) -> StateLoad<B>
    where
        F: FnOnce(T) -> B,
    {
        StateLoad::new(f(self.data), self.is_cold)
    }
}

/// Result of the account load from Journal state
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AccountLoad {
    /// Is account empty, if `true` account is not created
    pub is_empty: bool,
    /// Account address and id.
    pub address_and_id: AddressAndId,
    /// Does account have delegate code and delegated account is cold loaded
    pub delegated_account_address: Option<StateLoad<AddressAndId>>,
}

impl AccountLoad {
    /// Returns the bytecode address.
    ///
    /// If the account has a delegated account, it returns the delegated account address.
    /// Otherwise, it returns the account address.
    #[inline]
    pub fn bytecode_address(&self) -> AddressAndId {
        if let Some(delegated_account_address) = &self.delegated_account_address {
            delegated_account_address.data
        } else {
            self.address_and_id
        }
    }

    /// Returns whether the account is delegated.
    #[inline]
    pub fn is_delegated(&self) -> bool {
        self.delegated_account_address.is_some()
    }
}
