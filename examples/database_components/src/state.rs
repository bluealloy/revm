//! State database component from [`crate::Database`]

use auto_impl::auto_impl;
use core::ops::Deref;
use revm::{
    primitives::{Address, StorageKey, StorageValue, B256},
    state::{AccountInfo, Bytecode},
};
use std::sync::Arc;

/// Trait for mutable access to state data including accounts, code, and storage.
/// This is typically used for database implementations that may modify state
/// or need mutable access for caching purposes.
#[auto_impl(&mut, Box)]
pub trait State {
    /// Error type for state operations
    type Error: core::error::Error + Send + Sync + 'static;

    /// Gets basic account information.
    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error>;

    /// Gets account code by its hash.
    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error>;

    /// Gets storage value of address at index.
    fn storage(&mut self, address: Address, index: StorageKey)
        -> Result<StorageValue, Self::Error>;
}

/// Trait for immutable access to state data including accounts, code, and storage.
/// This is typically used for read-only database implementations or when
/// state data is pre-loaded and doesn't require modification.
#[auto_impl(&, &mut, Box, Rc, Arc)]
pub trait StateRef {
    /// Error type for state operations
    type Error: core::error::Error + Send + Sync + 'static;

    /// Gets basic account information.
    fn basic(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error>;

    /// Gets account code by its hash.
    fn code_by_hash(&self, code_hash: B256) -> Result<Bytecode, Self::Error>;

    /// Gets storage value of address at index.
    fn storage(&self, address: Address, index: StorageKey) -> Result<StorageValue, Self::Error>;
}

impl<T> State for &T
where
    T: StateRef,
{
    type Error = <T as StateRef>::Error;

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        StateRef::basic(*self, address)
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        StateRef::code_by_hash(*self, code_hash)
    }

    fn storage(
        &mut self,
        address: Address,
        index: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        StateRef::storage(*self, address, index)
    }
}

impl<T> State for Arc<T>
where
    T: StateRef,
{
    type Error = <T as StateRef>::Error;

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        self.deref().basic(address)
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.deref().code_by_hash(code_hash)
    }

    fn storage(
        &mut self,
        address: Address,
        index: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        self.deref().storage(address, index)
    }
}
