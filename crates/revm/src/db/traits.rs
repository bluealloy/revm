use crate::Account;
use hashbrown::HashMap as Map;

use primitive_types::{H160, H256, U256};

use crate::AccountInfo;
use auto_impl::auto_impl;
use bytes::Bytes;

#[auto_impl(& mut, Box)]
pub trait Database {
    /// Whether account at address exists.
    fn exists(&mut self, address: H160) -> Option<AccountInfo>;
    /// Get basic account information.
    fn basic(&mut self, address: H160) -> AccountInfo;
    /// Get account code by its hash
    fn code_by_hash(&mut self, code_hash: H256) -> Bytes;
    /// Get storage value of address at index.
    fn storage(&mut self, address: H160, index: U256) -> U256;

    // History related
    fn block_hash(&mut self, number: U256) -> H256;
}

#[auto_impl(& mut, Box)]
pub trait DatabaseCommit {
    fn commit(&mut self, changes: Map<H160, Account>);
}

#[auto_impl(&, Box)]
pub trait DatabaseRef {
    /// Whether account at address exists.
    fn exists(&self, address: H160) -> Option<AccountInfo>;
    /// Get basic account information.
    fn basic(&self, address: H160) -> AccountInfo;
    /// Get account code by its hash
    fn code_by_hash(&self, code_hash: H256) -> Bytes;
    /// Get storage value of address at index.
    fn storage(&self, address: H160, index: U256) -> U256;

    // History related
    fn block_hash(&self, number: U256) -> H256;
}

pub struct RefDBWrapper<'a> {
    pub db: &'a dyn DatabaseRef,
}

impl<'a> RefDBWrapper<'a> {
    pub fn new(db: &'a dyn DatabaseRef) -> Self {
        Self { db }
    }
}

impl<'a> Database for RefDBWrapper<'a> {
    /// Whether account at address exists.
    fn exists(&mut self, address: H160) -> Option<AccountInfo> {
        self.db.exists(address)
    }
    /// Get basic account information.
    fn basic(&mut self, address: H160) -> AccountInfo {
        self.db.basic(address)
    }
    /// Get account code by its hash
    fn code_by_hash(&mut self, code_hash: H256) -> Bytes {
        self.db.code_by_hash(code_hash)
    }
    /// Get storage value of address at index.
    fn storage(&mut self, address: H160, index: U256) -> U256 {
        self.db.storage(address, index)
    }

    // History related
    fn block_hash(&mut self, number: U256) -> H256 {
        self.db.block_hash(number)
    }
}
