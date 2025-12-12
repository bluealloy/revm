//! Database interface.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

use core::convert::Infallible;

use auto_impl::auto_impl;
use primitives::{address, Address, HashMap, StorageKey, StorageValue, B256, U256};
use state::{Account, AccountInfo, Bytecode};
use std::vec::Vec;

/// Address with all `0xff..ff` in it. Used for testing.
pub const FFADDRESS: Address = address!("0xffffffffffffffffffffffffffffffffffffffff");
/// BENCH_TARGET address
pub const BENCH_TARGET: Address = FFADDRESS;
/// Common test balance used for benchmark addresses
pub const TEST_BALANCE: U256 = U256::from_limbs([10_000_000_000_000_000, 0, 0, 0]);
/// BENCH_TARGET_BALANCE balance
pub const BENCH_TARGET_BALANCE: U256 = TEST_BALANCE;
/// Address with all `0xee..ee` in it. Used for testing.
pub const EEADDRESS: Address = address!("0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee");
/// BENCH_CALLER address
pub const BENCH_CALLER: Address = EEADDRESS;
/// BENCH_CALLER_BALANCE balance
pub const BENCH_CALLER_BALANCE: U256 = TEST_BALANCE;

#[cfg(feature = "asyncdb")]
pub mod async_db;
pub mod bal;
pub mod either;
pub mod empty_db;
pub mod erased_error;
pub mod try_commit;

#[cfg(feature = "asyncdb")]
pub use async_db::{DatabaseAsync, WrapDatabaseAsync};
pub use empty_db::{EmptyDB, EmptyDBTyped};
pub use erased_error::ErasedError;
pub use try_commit::{ArcUpgradeError, TryDatabaseCommit};

/// Database error marker is needed to implement From conversion for Error type.
pub trait DBErrorMarker: core::error::Error + Send + Sync + 'static {}

/// Implement marker for `()`.
impl DBErrorMarker for Infallible {}
impl DBErrorMarker for ErasedError {}

/// EVM database interface.
#[auto_impl(&mut, Box)]
pub trait Database {
    /// The database error type.
    type Error: DBErrorMarker;

    /// Gets basic account information.
    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error>;

    /// Gets account code by its hash.
    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error>;

    /// Gets storage value of address at index.
    fn storage(&mut self, address: Address, index: StorageKey)
        -> Result<StorageValue, Self::Error>;

    /// Gets storage value of account by its id.
    ///
    /// Default implementation is to call [`Database::storage`] method.
    #[inline]
    fn storage_by_account_id(
        &mut self,
        address: Address,
        account_id: usize,
        storage_key: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        let _ = account_id;
        self.storage(address, storage_key)
    }

    /// Gets block hash by block number.
    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error>;
}

/// EVM database commit interface.
#[auto_impl(&mut, Box)]
pub trait DatabaseCommit {
    /// Commit changes to the database.
    fn commit(&mut self, changes: HashMap<Address, Account>);

    /// Commit changes to the database with an iterator.
    ///
    /// Implementors of [`DatabaseCommit`] should override this method when possible for efficiency.
    ///
    /// Callers should prefer using [`DatabaseCommit::commit`] when they already have a [`HashMap`].
    fn commit_iter(&mut self, changes: impl IntoIterator<Item = (Address, Account)>) {
        let changes: HashMap<Address, Account> = changes.into_iter().collect();
        self.commit(changes);
    }
}

/// EVM database interface.
///
/// Contains the same methods as [`Database`], but with `&self` receivers instead of `&mut self`.
///
/// Use [`WrapDatabaseRef`] to provide [`Database`] implementation for a type
/// that only implements this trait.
#[auto_impl(&, &mut, Box, Rc, Arc)]
pub trait DatabaseRef {
    /// The database error type.
    type Error: DBErrorMarker;

    /// Gets basic account information.
    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error>;

    /// Gets account code by its hash.
    fn code_by_hash_ref(&self, code_hash: B256) -> Result<Bytecode, Self::Error>;

    /// Gets storage value of address at index.
    fn storage_ref(&self, address: Address, index: StorageKey)
        -> Result<StorageValue, Self::Error>;

    /// Gets storage value of account by its id.
    ///
    /// Default implementation is to call [`DatabaseRef::storage_ref`] method.
    #[inline]
    fn storage_by_account_id_ref(
        &self,
        address: Address,
        account_id: usize,
        storage_key: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        let _ = account_id;
        self.storage_ref(address, storage_key)
    }

    /// Gets block hash by block number.
    fn block_hash_ref(&self, number: u64) -> Result<B256, Self::Error>;
}

/// Wraps a [`DatabaseRef`] to provide a [`Database`] implementation.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WrapDatabaseRef<T: DatabaseRef>(pub T);

impl<F: DatabaseRef> From<F> for WrapDatabaseRef<F> {
    #[inline]
    fn from(f: F) -> Self {
        WrapDatabaseRef(f)
    }
}

impl<T: DatabaseRef> Database for WrapDatabaseRef<T> {
    type Error = T::Error;

    #[inline]
    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        self.0.basic_ref(address)
    }

    #[inline]
    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.0.code_by_hash_ref(code_hash)
    }

    #[inline]
    fn storage(
        &mut self,
        address: Address,
        index: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        self.0.storage_ref(address, index)
    }

    #[inline]
    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        self.0.block_hash_ref(number)
    }
}

impl<T: DatabaseRef + DatabaseCommit> DatabaseCommit for WrapDatabaseRef<T> {
    #[inline]
    fn commit(&mut self, changes: HashMap<Address, Account>) {
        self.0.commit(changes)
    }
}

impl<T: DatabaseRef> DatabaseRef for WrapDatabaseRef<T> {
    type Error = T::Error;

    #[inline]
    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        self.0.basic_ref(address)
    }

    #[inline]
    fn code_by_hash_ref(&self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.0.code_by_hash_ref(code_hash)
    }

    #[inline]
    fn storage_ref(
        &self,
        address: Address,
        index: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        self.0.storage_ref(address, index)
    }

    #[inline]
    fn block_hash_ref(&self, number: u64) -> Result<B256, Self::Error> {
        self.0.block_hash_ref(number)
    }
}

impl<T: Database + DatabaseCommit> DatabaseCommitExt for T {
    // default implementation
}

/// EVM database commit interface.
pub trait DatabaseCommitExt: Database + DatabaseCommit {
    /// Iterates over received balances and increment all account balances.
    ///
    /// Update will create transitions for all accounts that are updated.
    fn increment_balances(
        &mut self,
        balances: impl IntoIterator<Item = (Address, u128)>,
    ) -> Result<(), Self::Error> {
        // Make transition and update cache state
        let transitions = balances
            .into_iter()
            .map(|(address, balance)| {
                let mut original_account = match self.basic(address)? {
                    Some(acc_info) => Account::from(acc_info),
                    None => Account::new_not_existing(0),
                };
                original_account.info.balance = original_account
                    .info
                    .balance
                    .saturating_add(U256::from(balance));
                original_account.mark_touch();
                Ok((address, original_account))
            })
            // Unfortunately must collect here to short circuit on error
            .collect::<Result<Vec<_>, _>>()?;

        self.commit_iter(transitions);
        Ok(())
    }

    /// Drains balances from given account and return those values.
    ///
    /// It is used for DAO hardfork state change to move values from given accounts.
    fn drain_balances(
        &mut self,
        addresses: impl IntoIterator<Item = Address>,
    ) -> Result<Vec<u128>, Self::Error> {
        // Make transition and update cache state
        let addresses_iter = addresses.into_iter();
        let (lower, _) = addresses_iter.size_hint();
        let mut transitions = Vec::with_capacity(lower);
        let balances = addresses_iter
            .map(|address| {
                let mut original_account = match self.basic(address)? {
                    Some(acc_info) => Account::from(acc_info),
                    None => Account::new_not_existing(0),
                };
                let balance = core::mem::take(&mut original_account.info.balance);
                original_account.mark_touch();
                transitions.push((address, original_account));
                Ok(balance.try_into().unwrap())
            })
            .collect::<Result<Vec<_>, _>>()?;

        self.commit_iter(transitions);
        Ok(balances)
    }
}
