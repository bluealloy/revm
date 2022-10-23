mod in_memory_db;

#[cfg(feature = "web3db")]
pub mod web3db;
#[cfg(feature = "web3db")]
pub use web3db::Web3DB;

pub use in_memory_db::{AccountState, BenchmarkDB, CacheDB, DbAccount, EmptyDB, InMemoryDB};

use crate::{interpreter::bytecode::Bytecode, Account};
use hashbrown::HashMap as Map;
use primitive_types::{H160, H256};
use ruint::aliases::U256;

use crate::AccountInfo;
use auto_impl::auto_impl;

#[auto_impl(& mut, Box)]
pub trait Database {
    type Error;
    /// Get basic account information.
    fn basic(&mut self, address: H160) -> Result<Option<AccountInfo>, Self::Error>;
    /// Get account code by its hash
    fn code_by_hash(&mut self, code_hash: H256) -> Result<Bytecode, Self::Error>;
    /// Get storage value of address at index.
    fn storage(&mut self, address: H160, index: U256) -> Result<U256, Self::Error>;

    // History related
    fn block_hash(&mut self, number: U256) -> Result<H256, Self::Error>;
}

#[auto_impl(& mut, Box)]
pub trait DatabaseCommit {
    fn commit(&mut self, changes: Map<H160, Account>);
}

#[auto_impl(&, Box)]
pub trait DatabaseRef {
    type Error;
    /// Whether account at address exists.
    //fn exists(&self, address: H160) -> Option<AccountInfo>;
    /// Get basic account information.
    fn basic(&self, address: H160) -> Result<Option<AccountInfo>, Self::Error>;
    /// Get account code by its hash
    fn code_by_hash(&self, code_hash: H256) -> Result<Bytecode, Self::Error>;
    /// Get storage value of address at index.
    fn storage(&self, address: H160, index: U256) -> Result<U256, Self::Error>;

    // History related
    fn block_hash(&self, number: U256) -> Result<H256, Self::Error>;
}

pub struct RefDBWrapper<'a, Error> {
    pub db: &'a dyn DatabaseRef<Error = Error>,
}

impl<'a, Error> RefDBWrapper<'a, Error> {
    pub fn new(db: &'a dyn DatabaseRef<Error = Error>) -> Self {
        Self { db }
    }
}

impl<'a, Error> Database for RefDBWrapper<'a, Error> {
    type Error = Error;
    /// Get basic account information.
    fn basic(&mut self, address: H160) -> Result<Option<AccountInfo>, Self::Error> {
        self.db.basic(address)
    }
    /// Get account code by its hash
    fn code_by_hash(&mut self, code_hash: H256) -> Result<Bytecode, Self::Error> {
        self.db.code_by_hash(code_hash)
    }
    /// Get storage value of address at index.
    fn storage(&mut self, address: H160, index: U256) -> Result<U256, Self::Error> {
        self.db.storage(address, index)
    }

    // History related
    fn block_hash(&mut self, number: U256) -> Result<H256, Self::Error> {
        self.db.block_hash(number)
    }
}
