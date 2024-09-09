use crate::{Account, AccountInfo, Address, Bytecode, HashMap, B256, U256};
use auto_impl::auto_impl;

pub mod components;
pub mod emptydb;

pub use components::{
    BlockHash, BlockHashRef, DatabaseComponentError, DatabaseComponents, State, StateRef,
};
pub use emptydb::{EmptyDB, EmptyDBTyped};

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
