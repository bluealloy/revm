use crate::context::{SStoreResult, SelfDestructResult};
use core::ops::{Deref, DerefMut};
use database_interface::Database;
use primitives::{
    hardfork::SpecId, Address, Bytes, HashSet, Log, StorageKey, StorageValue, B256, U256,
};
use state::{
    bytecode::{EOF_MAGIC_BYTES, EOF_MAGIC_HASH},
    Account, Bytecode,
};

/// Trait that contains database and journal of all changes that were made to the state.
pub trait JournalTr {
    type Database: Database;
    type FinalOutput;

    /// Creates new Journaled state.
    ///
    /// Dont forget to set spec_id.
    fn new(database: Self::Database) -> Self;

    /// Returns the database.
    fn db_ref(&self) -> &Self::Database;

    /// Returns the mutable database.
    fn db(&mut self) -> &mut Self::Database;

    /// Returns the storage value from Journal state.
    ///
    /// Loads the storage from database if not found in Journal state.
    fn sload(
        &mut self,
        address: Address,
        key: StorageKey,
    ) -> Result<StateLoad<StorageValue>, <Self::Database as Database>::Error>;

    /// Stores the storage value in Journal state.
    fn sstore(
        &mut self,
        address: Address,
        key: StorageKey,
        value: StorageValue,
    ) -> Result<StateLoad<SStoreResult>, <Self::Database as Database>::Error>;

    /// Loads transient storage value.
    fn tload(&mut self, address: Address, key: StorageKey) -> StorageValue;

    /// Stores transient storage value.
    fn tstore(&mut self, address: Address, key: StorageKey, value: StorageValue);

    /// Logs the log in Journal state.
    fn log(&mut self, log: Log);

    /// Marks the account for selfdestruction and transfers all the balance to the target.
    fn selfdestruct(
        &mut self,
        address: Address,
        target: Address,
    ) -> Result<StateLoad<SelfDestructResult>, <Self::Database as Database>::Error>;

    /// Warms the account and storage.
    fn warm_account_and_storage(
        &mut self,
        address: Address,
        storage_keys: impl IntoIterator<Item = StorageKey>,
    ) -> Result<(), <Self::Database as Database>::Error>;

    /// Warms the account.
    fn warm_account(&mut self, address: Address);

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

    /// Increments the nonce of the account.
    fn inc_account_nonce(
        &mut self,
        address: Address,
    ) -> Result<Option<u64>, <Self::Database as Database>::Error>;

    /// Loads the account.
    fn load_account(
        &mut self,
        address: Address,
    ) -> Result<StateLoad<&mut Account>, <Self::Database as Database>::Error>;

    /// Loads the account code.
    fn load_account_code(
        &mut self,
        address: Address,
    ) -> Result<StateLoad<&mut Account>, <Self::Database as Database>::Error>;

    /// Loads the account delegated.
    fn load_account_delegated(
        &mut self,
        address: Address,
    ) -> Result<StateLoad<AccountLoad>, <Self::Database as Database>::Error>;

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
    ///
    /// In case of EOF account it will return `EOF_MAGIC` (0xEF00) as code.
    #[inline]
    fn code(
        &mut self,
        address: Address,
    ) -> Result<StateLoad<Bytes>, <Self::Database as Database>::Error> {
        let a = self.load_account_code(address)?;
        // SAFETY: Safe to unwrap as load_code will insert code if it is empty.
        let code = a.info.code.as_ref().unwrap();

        let code = if code.is_eof() {
            EOF_MAGIC_BYTES.clone()
        } else {
            code.original_bytes()
        };

        Ok(StateLoad::new(code, a.is_cold))
    }

    /// Gets code hash of account.
    ///
    /// In case of EOF account it will return `EOF_MAGIC_HASH`
    /// (the hash of `0xEF00`).
    fn code_hash(
        &mut self,
        address: Address,
    ) -> Result<StateLoad<B256>, <Self::Database as Database>::Error> {
        let acc = self.load_account_code(address)?;
        if acc.is_empty() {
            return Ok(StateLoad::new(B256::ZERO, acc.is_cold));
        }
        // SAFETY: Safe to unwrap as load_code will insert code if it is empty.
        let code = acc.info.code.as_ref().unwrap();

        let hash = if code.is_eof() {
            EOF_MAGIC_HASH
        } else {
            acc.info.code_hash
        };

        Ok(StateLoad::new(hash, acc.is_cold))
    }

    /// Called at the end of the transaction to clean all residue data from journal.
    fn clear(&mut self);

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

    /// Does cleanup and returns modified state.
    ///
    /// This resets the [JournalTr] to its initial state.
    fn finalize(&mut self) -> Self::FinalOutput;
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
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JournalCheckpoint {
    pub log_i: usize,
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
    /// Does account have delegate code and delegated account is cold loaded
    pub is_delegate_account_cold: Option<bool>,
    /// Is account empty, if `true` account is not created
    pub is_empty: bool,
}
