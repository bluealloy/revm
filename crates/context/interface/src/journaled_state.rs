use core::ops::{Deref, DerefMut};
use database_interface::{Database, DatabaseGetter};
use primitives::{Address, Log, B256, U256};
use specification::hardfork::SpecId;
use state::{Account, Bytecode};
use std::boxed::Box;

use crate::host::{SStoreResult, SelfDestructResult};

pub trait Journal {
    type Database: Database;
    type FinalOutput;

    /// Creates new Journaled state.
    ///
    /// Dont forget to set spec_id.
    fn new(database: Self::Database) -> Self;

    /// Returns the database.
    fn db(&self) -> &Self::Database;

    /// Returns the mutable database.
    fn db_mut(&mut self) -> &mut Self::Database;

    /// Returns the storage value from Journal state.
    ///
    /// Loads the storage from database if not found in Journal state.
    fn sload(
        &mut self,
        address: Address,
        key: U256,
    ) -> Result<StateLoad<U256>, <Self::Database as Database>::Error>;

    /// Stores the storage value in Journal state.
    fn sstore(
        &mut self,
        address: Address,
        key: U256,
        value: U256,
    ) -> Result<StateLoad<SStoreResult>, <Self::Database as Database>::Error>;

    /// Loads transient storage value.
    fn tload(&mut self, address: Address, key: U256) -> U256;

    /// Stores transient storage value.
    fn tstore(&mut self, address: Address, key: U256, value: U256);

    /// Logs the log in Journal state.
    fn log(&mut self, log: Log);

    /// Marks the account for selfdestruction and transfers all the balance to the target.
    fn selfdestruct(
        &mut self,
        address: Address,
        target: Address,
    ) -> Result<StateLoad<SelfDestructResult>, <Self::Database as Database>::Error>;

    fn warm_account_and_storage(
        &mut self,
        address: Address,
        storage_keys: impl IntoIterator<Item = U256>,
    ) -> Result<(), <Self::Database as Database>::Error>;

    fn warm_account(&mut self, address: Address);

    fn set_spec_id(&mut self, spec_id: SpecId);

    fn touch_account(&mut self, address: Address);

    /// TODO instruction result is not known
    fn transfer(
        &mut self,
        from: &Address,
        to: &Address,
        balance: U256,
    ) -> Result<Option<TransferError>, <Self::Database as Database>::Error>;

    fn inc_account_nonce(
        &mut self,
        address: Address,
    ) -> Result<Option<u64>, <Self::Database as Database>::Error>;

    fn load_account(
        &mut self,
        address: Address,
    ) -> Result<StateLoad<&mut Account>, <Self::Database as Database>::Error>;

    fn load_account_code(
        &mut self,
        address: Address,
    ) -> Result<StateLoad<&mut Account>, <Self::Database as Database>::Error>;

    fn load_account_delegated(
        &mut self,
        address: Address,
    ) -> Result<AccountLoad, <Self::Database as Database>::Error>;

    /// Set bytecode with hash. Assume that account is warm.
    fn set_code_with_hash(&mut self, address: Address, code: Bytecode, hash: B256);

    /// Assume account is warm
    #[inline]
    fn set_code(&mut self, address: Address, code: Bytecode) {
        let hash = code.hash_slow();
        self.set_code_with_hash(address, code, hash);
    }

    /// Called at the end of the transaction to clean all residue data from journal.
    fn clear(&mut self);

    fn checkpoint(&mut self) -> JournalCheckpoint;

    fn checkpoint_commit(&mut self);

    fn checkpoint_revert(&mut self, checkpoint: JournalCheckpoint);

    fn create_account_checkpoint(
        &mut self,
        caller: Address,
        address: Address,
        balance: U256,
        spec_id: SpecId,
    ) -> Result<JournalCheckpoint, TransferError>;

    fn depth(&self) -> usize;

    /// Does cleanup and returns modified state.
    ///
    /// This resets the [Journal] to its initial state.
    fn finalize(&mut self) -> Result<Self::FinalOutput, <Self::Database as Database>::Error>;
}

/// Transfer and creation result.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TransferError {
    /// Caller does not have enough funds
    OutOfFunds,
    /// Overflow in target account.
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

/// State load information that contains the data and if the account or storage is cold loaded.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StateLoad<T> {
    /// returned data
    pub data: T,
    /// True if account is cold loaded.
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

/// Result of the account load from Journal state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AccountLoad {
    /// Is account and delegate code are loaded
    pub load: Eip7702CodeLoad<()>,
    /// Is account empty, if true account is not created.
    pub is_empty: bool,
}

impl Deref for AccountLoad {
    type Target = Eip7702CodeLoad<()>;

    fn deref(&self) -> &Self::Target {
        &self.load
    }
}

impl DerefMut for AccountLoad {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.load
    }
}

/// EIP-7702 code load result that contains optional delegation is_cold information.
///
/// [`Self::is_delegate_account_cold`] will be [`Some`] if account has delegation.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Eip7702CodeLoad<T> {
    /// returned data
    pub state_load: StateLoad<T>,
    /// True if account has delegate code and delegated account is cold loaded.
    pub is_delegate_account_cold: Option<bool>,
}

impl<T> Deref for Eip7702CodeLoad<T> {
    type Target = StateLoad<T>;

    fn deref(&self) -> &Self::Target {
        &self.state_load
    }
}

impl<T> DerefMut for Eip7702CodeLoad<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state_load
    }
}

impl<T> Eip7702CodeLoad<T> {
    /// Returns a new [`Eip7702CodeLoad`] with the given data and without delegation.
    pub fn new_state_load(state_load: StateLoad<T>) -> Self {
        Self {
            state_load,
            is_delegate_account_cold: None,
        }
    }

    /// Returns a new [`Eip7702CodeLoad`] with the given data and without delegation.
    pub fn new_not_delegated(data: T, is_cold: bool) -> Self {
        Self {
            state_load: StateLoad::new(data, is_cold),
            is_delegate_account_cold: None,
        }
    }

    /// Deconstructs the [`Eip7702CodeLoad`] by extracting data and
    /// returning a new [`Eip7702CodeLoad`] with empty data.
    pub fn into_components(self) -> (T, Eip7702CodeLoad<()>) {
        let is_cold = self.is_cold;
        (
            self.state_load.data,
            Eip7702CodeLoad {
                state_load: StateLoad::new((), is_cold),
                is_delegate_account_cold: self.is_delegate_account_cold,
            },
        )
    }

    /// Sets the delegation cold load status.
    pub fn set_delegate_load(&mut self, is_delegate_account_cold: bool) {
        self.is_delegate_account_cold = Some(is_delegate_account_cold);
    }

    /// Returns a new [`Eip7702CodeLoad`] with the given data and delegation cold load status.
    pub fn new(state_load: StateLoad<T>, is_delegate_account_cold: bool) -> Self {
        Self {
            state_load,
            is_delegate_account_cold: Some(is_delegate_account_cold),
        }
    }
}

/// Helper that extracts database error from [`JournalStateGetter`].
pub type JournalStateGetterDBError<CTX> =
    <<<CTX as JournalStateGetter>::Journal as Journal>::Database as Database>::Error;

pub trait JournalStateGetter: DatabaseGetter {
    type Journal: Journal<Database = <Self as DatabaseGetter>::Database>;

    fn journal(&mut self) -> &mut Self::Journal;
}

impl<T: JournalStateGetter> JournalStateGetter for &mut T {
    type Journal = T::Journal;

    fn journal(&mut self) -> &mut Self::Journal {
        T::journal(*self)
    }
}

impl<T: JournalStateGetter> JournalStateGetter for Box<T> {
    type Journal = T::Journal;

    fn journal(&mut self) -> &mut Self::Journal {
        T::journal(self.as_mut())
    }
}
