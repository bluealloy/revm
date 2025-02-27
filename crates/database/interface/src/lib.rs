//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

use auto_impl::auto_impl;
use primitives::{Address, HashMap, B256, U256};
use state::{Account, AccountInfo, Bytecode};

#[cfg(feature = "asyncdb")]
pub mod async_db;
pub mod empty_db;

#[cfg(feature = "asyncdb")]
pub use async_db::{DatabaseAsync, WrapDatabaseAsync};
pub use empty_db::{EmptyDB, EmptyDBTyped};

/// EVM database interface.
#[auto_impl(&mut, Box)]
pub trait Database {
    /// The database error type.
    type Error;

    /// Get basic account information.
    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error>;

    /// Get account code by its hash.
    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error>;

    /// Get storage value of address at index.
    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error>;

    /// Get block hash by block number.
    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error>;
}

/// EVM database commit interface.
#[auto_impl(&mut, Box)]
pub trait DatabaseCommit {
    /// Commit changes to the database.
    fn commit(&mut self, changes: HashMap<Address, Account>);
}

/// EVM database commit interface that can fail.
pub trait TryDatabaseCommit {
    /// Error type to be thrown when state accumulation fails.
    type Error: core::error::Error;

    /// Commit changes to the database.
    fn try_commit(&mut self, changes: HashMap<Address, Account>) -> Result<(), Self::Error>;
}

impl<Db> TryDatabaseCommit for Db
where
    Db: DatabaseCommit,
{
    type Error = core::convert::Infallible;

    #[inline]
    fn try_commit(&mut self, changes: HashMap<Address, Account>) -> Result<(), Self::Error> {
        self.commit(changes);
        Ok(())
    }
}

#[cfg(feature = "std")]
/// Error type for implementation of [`TryDatabaseCommit`] on
/// [`std::sync::Arc`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArcUpgradeError;

#[cfg(feature = "std")]
impl core::fmt::Display for ArcUpgradeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Arc reference is not unique, cannot mutate")
    }
}

#[cfg(feature = "std")]
impl core::error::Error for ArcUpgradeError {}

#[cfg(feature = "std")]
impl<Db> TryDatabaseCommit for std::sync::Arc<Db>
where
    Db: DatabaseCommit + Send + Sync,
{
    type Error = ArcUpgradeError;

    #[inline]
    fn try_commit(&mut self, changes: HashMap<Address, Account>) -> Result<(), Self::Error> {
        std::sync::Arc::get_mut(self)
            .map(|db| db.commit(changes))
            .ok_or(ArcUpgradeError)
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
    type Error;

    /// Get basic account information.
    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error>;

    /// Get account code by its hash.
    fn code_by_hash_ref(&self, code_hash: B256) -> Result<Bytecode, Self::Error>;

    /// Get storage value of address at index.
    fn storage_ref(&self, address: Address, index: U256) -> Result<U256, Self::Error>;

    /// Get block hash by block number.
    fn block_hash_ref(&self, number: u64) -> Result<B256, Self::Error>;
}

/// Wraps a [`DatabaseRef`] to provide a [`Database`] implementation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
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
    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
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
