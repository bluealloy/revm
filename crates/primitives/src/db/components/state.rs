//! State database component from [`crate::db::Database`]
//! it is used inside [`crate::db::DatabaseComponents`]

use crate::{AccountInfo, Address, Bytecode, B256, U256};
use auto_impl::auto_impl;
use core::ops::Deref;
use std::sync::Arc;

#[auto_impl(&mut, Box)]
pub trait State {
    type Error;

    /// Get basic account information.
    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error>;

    /// Get account code by its hash
    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error>;

    /// Get storage value of address at index.
    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error>;
}

#[auto_impl(&, &mut, Box, Rc, Arc)]
pub trait StateRef {
    type Error;

    /// Get basic account information.
    fn basic(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error>;

    /// Get account code by its hash
    fn code_by_hash(&self, code_hash: B256) -> Result<Bytecode, Self::Error>;

    /// Get storage value of address at index.
    fn storage(&self, address: Address, index: U256) -> Result<U256, Self::Error>;
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

    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
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

    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
        self.deref().storage(address, index)
    }
}
