pub mod components;

use crate::AccountInfo;
use crate::U256;
use crate::{Account, Bytecode};
use crate::{B160, B256};
use auto_impl::auto_impl;
use hashbrown::HashMap as Map;

pub use components::{
    BlockHash, BlockHashRef, DatabaseComponentError, DatabaseComponents, State, StateRef,
};

#[auto_impl(&mut, Box)]
pub trait Database {
    type Error;
    /// Get basic account information.
    fn basic(&mut self, address: B160) -> Result<Option<AccountInfo>, Self::Error>;
    /// Get account code by its hash
    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error>;
    /// Get storage value of address at index.
    fn storage(&mut self, address: B160, index: U256) -> Result<U256, Self::Error>;

    // History related
    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error>;
}

impl<F: DatabaseRef> From<F> for WrapDatabaseRef<F> {
    fn from(f: F) -> Self {
        WrapDatabaseRef(f)
    }
}

#[auto_impl(&mut, Box)]
pub trait DatabaseCommit {
    fn commit(&mut self, changes: Map<B160, Account>);
}

#[auto_impl(&, Box, Arc)]
pub trait DatabaseRef {
    type Error;
    /// Whether account at address exists.
    //fn exists(&self, address: B160) -> Option<AccountInfo>;
    /// Get basic account information.
    fn basic(&self, address: B160) -> Result<Option<AccountInfo>, Self::Error>;
    /// Get account code by its hash
    fn code_by_hash(&self, code_hash: B256) -> Result<Bytecode, Self::Error>;
    /// Get storage value of address at index.
    fn storage(&self, address: B160, index: U256) -> Result<U256, Self::Error>;

    // History related
    fn block_hash(&self, number: U256) -> Result<B256, Self::Error>;
}

pub struct WrapDatabaseRef<T: DatabaseRef>(pub T);

impl<T: DatabaseRef> Database for WrapDatabaseRef<T> {
    type Error = T::Error;

    fn basic(&mut self, address: B160) -> Result<Option<AccountInfo>, Self::Error> {
        self.0.basic(address)
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.0.code_by_hash(code_hash)
    }

    fn storage(&mut self, address: B160, index: U256) -> Result<U256, Self::Error> {
        self.0.storage(address, index)
    }

    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error> {
        self.0.block_hash(number)
    }
}

/// Wraps a `dyn DatabaseRef` to provide a [`Database`] implementation.
#[doc(hidden)]
#[deprecated = "use `WrapDatabaseRef` instead"]
pub struct RefDBWrapper<'a, E> {
    pub db: &'a dyn DatabaseRef<Error = E>,
}

#[allow(deprecated)]
impl<'a, E> RefDBWrapper<'a, E> {
    #[inline]
    pub fn new(db: &'a dyn DatabaseRef<Error = E>) -> Self {
        Self { db }
    }
}

#[allow(deprecated)]
impl<'a, E> Database for RefDBWrapper<'a, E> {
    type Error = E;

    #[inline]
    fn basic(&mut self, address: B160) -> Result<Option<AccountInfo>, Self::Error> {
        self.db.basic(address)
    }

    #[inline]
    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.db.code_by_hash(code_hash)
    }

    #[inline]
    fn storage(&mut self, address: B160, index: U256) -> Result<U256, Self::Error> {
        self.db.storage(address, index)
    }

    #[inline]
    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error> {
        self.db.block_hash(number)
    }
}
