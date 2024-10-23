use core::ops::{Deref, DerefMut};
use database_interface::Database;
use primitives::{Address, B256, U256};
use specification::hardfork::SpecId;
use state::{Account, Bytecode};

pub trait JournaledState {
    type Database: Database;
    type Checkpoint;
    type FinalOutput;

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
    ) -> Result<Option<()>, <Self::Database as Database>::Error>;

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

    fn checkpoint(&mut self) -> Self::Checkpoint;

    fn checkpoint_commit(&mut self);

    fn checkpoint_revert(&mut self, checkpoint: Self::Checkpoint);

    /// Does cleanup and returns modified state.
    ///
    /// This resets the [JournaledState] to its initial state.
    fn finalize(&mut self) -> Result<Self::FinalOutput, <Self::Database as Database>::Error>;
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
