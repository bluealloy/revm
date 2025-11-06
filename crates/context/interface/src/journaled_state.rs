//! Journaled state trait [`JournalTr`] and related types.

pub mod account;
pub mod entry;

use crate::{
    context::{SStoreResult, SelfDestructResult},
    host::LoadError,
    journaled_state::{account::JournaledAccount, entry::JournalEntryTr},
};
use core::ops::{Deref, DerefMut};
use database_interface::Database;
use primitives::{
    hardfork::SpecId, Address, Bytes, HashMap, HashSet, Log, StorageKey, StorageValue, B256, U256,
};
use state::{Account, AccountInfo, Bytecode};
use std::{borrow::Cow, vec::Vec};

/// Trait that contains database and journal of all changes that were made to the state.
pub trait JournalTr {
    /// Database type that is used in the journal.
    type Database: Database;
    /// State type that is returned by the journal after finalization.
    type State;
    /// Journal Entry type that is used in the journal.
    type JournalEntry: JournalEntryTr;

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
        address: Address,
        key: StorageKey,
    ) -> Result<StateLoad<StorageValue>, <Self::Database as Database>::Error> {
        // unwrapping is safe as we only can get DBError
        self.sload_skip_cold_load(address, key, false)
            .map_err(JournalLoadError::unwrap_db_error)
    }

    /// Loads the storage value from Journal state.
    fn sload_skip_cold_load(
        &mut self,
        _address: Address,
        _key: StorageKey,
        _skip_cold_load: bool,
    ) -> Result<StateLoad<StorageValue>, JournalLoadError<<Self::Database as Database>::Error>>;

    /// Stores the storage value in Journal state.
    fn sstore(
        &mut self,
        address: Address,
        key: StorageKey,
        value: StorageValue,
    ) -> Result<StateLoad<SStoreResult>, <Self::Database as Database>::Error> {
        // unwrapping is safe as we only can get DBError
        self.sstore_skip_cold_load(address, key, value, false)
            .map_err(JournalLoadError::unwrap_db_error)
    }

    /// Stores the storage value in Journal state.
    fn sstore_skip_cold_load(
        &mut self,
        _address: Address,
        _key: StorageKey,
        _value: StorageValue,
        _skip_cold_load: bool,
    ) -> Result<StateLoad<SStoreResult>, JournalLoadError<<Self::Database as Database>::Error>>;

    /// Loads transient storage value.
    fn tload(&mut self, address: Address, key: StorageKey) -> StorageValue;

    /// Stores transient storage value.
    fn tstore(&mut self, address: Address, key: StorageKey, value: StorageValue);

    /// Logs the log in Journal state.
    fn log(&mut self, log: Log);

    /// Take logs from journal.
    fn take_logs(&mut self) -> Vec<Log>;

    /// Returns the logs from journal.
    fn logs(&self) -> &[Log];

    /// Marks the account for selfdestruction and transfers all the balance to the target.
    fn selfdestruct(
        &mut self,
        address: Address,
        target: Address,
        skip_cold_load: bool,
    ) -> Result<StateLoad<SelfDestructResult>, JournalLoadError<<Self::Database as Database>::Error>>;

    /// Sets access list inside journal.
    fn warm_access_list(&mut self, access_list: HashMap<Address, HashSet<StorageKey>>);

    /// Warms the coinbase account.
    fn warm_coinbase_account(&mut self, address: Address);

    /// Warms the precompiles.
    fn warm_precompiles(&mut self, addresses: HashSet<Address>);

    /// Returns the addresses of the precompiles.
    fn precompile_addresses(&self) -> &HashSet<Address>;

    /// Sets the spec id.
    fn set_spec_id(&mut self, spec_id: SpecId);

    /// Touches the account.
    fn touch_account(&mut self, address: Address);

    /// Transfers the balance from one account to another.
    fn transfer(
        &mut self,
        from: Address,
        to: Address,
        balance: U256,
    ) -> Result<Option<TransferError>, <Self::Database as Database>::Error>;

    /// Transfers the balance from one account to another. Assume form and to are loaded.
    fn transfer_loaded(
        &mut self,
        from: Address,
        to: Address,
        balance: U256,
    ) -> Option<TransferError>;

    /// Increments the balance of the account.
    fn caller_accounting_journal_entry(
        &mut self,
        address: Address,
        old_balance: U256,
        bump_nonce: bool,
    );

    /// Increments the balance of the account.
    fn balance_incr(
        &mut self,
        address: Address,
        balance: U256,
    ) -> Result<(), <Self::Database as Database>::Error>;

    /// Increments the nonce of the account.
    fn nonce_bump_journal_entry(&mut self, address: Address);

    /// Loads the account.
    fn load_account(
        &mut self,
        address: Address,
    ) -> Result<StateLoad<&Account>, <Self::Database as Database>::Error>;

    /// Loads the account code, use `load_account_with_code` instead.
    #[inline]
    #[deprecated(note = "Use `load_account_with_code` instead")]
    fn load_account_code(
        &mut self,
        address: Address,
    ) -> Result<StateLoad<&Account>, <Self::Database as Database>::Error> {
        self.load_account_with_code(address)
    }

    /// Loads the account with code.
    fn load_account_with_code(
        &mut self,
        address: Address,
    ) -> Result<StateLoad<&Account>, <Self::Database as Database>::Error>;

    /// Loads the account delegated.
    fn load_account_delegated(
        &mut self,
        address: Address,
    ) -> Result<StateLoad<AccountLoad>, <Self::Database as Database>::Error>;

    /// Loads the journaled account.
    #[inline]
    fn load_account_mut(
        &mut self,
        address: Address,
    ) -> Result<
        StateLoad<JournaledAccount<'_, Self::JournalEntry>>,
        <Self::Database as Database>::Error,
    > {
        self.load_account_mut_optional_code(address, false)
    }

    /// Loads the journaled account.
    #[inline]
    fn load_account_with_code_mut(
        &mut self,
        address: Address,
    ) -> Result<
        StateLoad<JournaledAccount<'_, Self::JournalEntry>>,
        <Self::Database as Database>::Error,
    > {
        self.load_account_mut_optional_code(address, true)
    }

    /// Loads the journaled account.
    fn load_account_mut_optional_code(
        &mut self,
        address: Address,
        load_code: bool,
    ) -> Result<
        StateLoad<JournaledAccount<'_, Self::JournalEntry>>,
        <Self::Database as Database>::Error,
    >;

    /// Sets bytecode with hash. Assume that account is warm.
    fn set_code_with_hash(&mut self, address: Address, code: Bytecode, hash: B256);

    /// Sets bytecode and calculates hash.
    ///
    /// Assume account is warm.
    #[inline]
    fn set_code(&mut self, address: Address, code: Bytecode) {
        let hash = code.hash_slow();
        self.set_code_with_hash(address, code, hash);
    }

    /// Returns account code bytes and if address is cold loaded.
    #[inline]
    fn code(
        &mut self,
        address: Address,
    ) -> Result<StateLoad<Bytes>, <Self::Database as Database>::Error> {
        let a = self.load_account_with_code(address)?;
        // SAFETY: Safe to unwrap as load_code will insert code if it is empty.
        let code = a.info.code.as_ref().unwrap().original_bytes();

        Ok(StateLoad::new(code, a.is_cold))
    }

    /// Gets code hash of account.
    fn code_hash(
        &mut self,
        address: Address,
    ) -> Result<StateLoad<B256>, <Self::Database as Database>::Error> {
        let acc = self.load_account_with_code(address)?;
        if acc.is_empty() {
            return Ok(StateLoad::new(B256::ZERO, acc.is_cold));
        }
        let hash = acc.info.code_hash;
        Ok(StateLoad::new(hash, acc.is_cold))
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
        caller: Address,
        address: Address,
        balance: U256,
        spec_id: SpecId,
    ) -> Result<JournalCheckpoint, TransferError>;

    /// Returns the depth of the journal.
    fn depth(&self) -> usize;

    /// Commit current transaction journal and returns transaction logs.
    fn commit_tx(&mut self);

    /// Discard current transaction journal by removing journal entries and logs and incrementing the transaction id.
    ///
    /// This function is useful to discard intermediate state that is interrupted by error and it will not revert
    /// any already committed changes and it is safe to call it multiple times.
    fn discard_tx(&mut self);

    /// Clear current journal resetting it to initial state and return changes state.
    fn finalize(&mut self) -> Self::State;

    /// Loads the account info from Journal state.
    fn load_account_info_skip_cold_load(
        &mut self,
        _address: Address,
        _load_code: bool,
        _skip_cold_load: bool,
    ) -> Result<AccountInfoLoad<'_>, JournalLoadError<<Self::Database as Database>::Error>>;
}

/// Error that can happen when loading account info.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum JournalLoadError<E> {
    /// Database error.
    DBError(E),
    /// Cold load skipped.
    ColdLoadSkipped,
}

impl<E> JournalLoadError<E> {
    /// Returns true if the error is a database error.
    #[inline]
    pub fn is_db_error(&self) -> bool {
        matches!(self, JournalLoadError::DBError(_))
    }

    /// Returns true if the error is a cold load skipped.
    #[inline]
    pub fn is_cold_load_skipped(&self) -> bool {
        matches!(self, JournalLoadError::ColdLoadSkipped)
    }

    /// Takes the error if it is a database error.
    #[inline]
    pub fn take_db_error(self) -> Option<E> {
        if let JournalLoadError::DBError(e) = self {
            Some(e)
        } else {
            None
        }
    }

    /// Unwraps the error if it is a database error.
    #[inline]
    pub fn unwrap_db_error(self) -> E {
        if let JournalLoadError::DBError(e) = self {
            e
        } else {
            panic!("Expected DBError");
        }
    }

    /// Converts the error to a load error.
    #[inline]
    pub fn into_parts(self) -> (LoadError, Option<E>) {
        match self {
            JournalLoadError::DBError(e) => (LoadError::DBError, Some(e)),
            JournalLoadError::ColdLoadSkipped => (LoadError::ColdLoadSkipped, None),
        }
    }
}

impl<E> From<E> for JournalLoadError<E> {
    fn from(e: E) -> Self {
        JournalLoadError::DBError(e)
    }
}

impl<E> From<JournalLoadError<E>> for LoadError {
    fn from(e: JournalLoadError<E>) -> Self {
        match e {
            JournalLoadError::DBError(_) => LoadError::DBError,
            JournalLoadError::ColdLoadSkipped => LoadError::ColdLoadSkipped,
        }
    }
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
    #[inline]
    pub fn new(data: T, is_cold: bool) -> Self {
        Self { data, is_cold }
    }

    /// Maps the data of the [`StateLoad`] to a new value.
    ///
    /// Useful for transforming the data of the [`StateLoad`] without changing the cold load status.
    #[inline]
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
    /// Does account have delegate code and delegated account is cold loaded
    pub is_delegate_account_cold: Option<bool>,
    /// Is account empty, if `true` account is not created
    pub is_empty: bool,
}

/// Result of the account load from Journal state
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AccountInfoLoad<'a> {
    /// Account info
    pub account: Cow<'a, AccountInfo>,
    /// Is account cold loaded
    pub is_cold: bool,
    /// Is account empty, if `true` account is not created
    pub is_empty: bool,
}

impl<'a> AccountInfoLoad<'a> {
    /// Creates new [`AccountInfoLoad`] with the given account info, cold load status and empty status.
    pub fn new(account: &'a AccountInfo, is_cold: bool, is_empty: bool) -> Self {
        Self {
            account: Cow::Borrowed(account),
            is_cold,
            is_empty,
        }
    }

    /// Maps the account info of the [`AccountInfoLoad`] to a new [`StateLoad`].
    ///
    /// Useful for transforming the account info of the [`AccountInfoLoad`] and preserving the cold load status.
    pub fn into_state_load<F, O>(self, f: F) -> StateLoad<O>
    where
        F: FnOnce(Cow<'a, AccountInfo>) -> O,
    {
        StateLoad::new(f(self.account), self.is_cold)
    }
}

impl<'a> Deref for AccountInfoLoad<'a> {
    type Target = AccountInfo;

    fn deref(&self) -> &Self::Target {
        &self.account
    }
}
