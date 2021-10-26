use primitive_types::{H160, H256, U256};

use crate::AccountInfo;
use bytes::Bytes;

pub trait Database {
    /// Whether account at address exists.
    fn exists(&mut self, address: H160) -> Option<AccountInfo>;
    /// Get basic account information.
    fn basic(&mut self, address: H160) -> AccountInfo;
    /// Get account code by its hash
    fn code_by_hash(&mut self, code_hash: H256) -> Bytes;
    /// Get storage value of address at index.
    fn storage(&mut self, address: H160, index: H256) -> H256;

    // History related
    fn block_hash(&mut self, number: U256) -> H256;

    fn account_mut(&mut self, address: H160) -> &mut AccountInfo;
}

impl<'a, D: Database> Database for &'a mut D {
    fn exists(&mut self, address: H160) -> Option<AccountInfo> {
        (*self).exists(address)
    }

    fn basic(&mut self, address: H160) -> AccountInfo {
        (*self).basic(address)
    }

    fn code_by_hash(&mut self, code_hash: H256) -> Bytes {
        (*self).code_by_hash(code_hash)
    }

    fn storage(&mut self, address: H160, index: H256) -> H256 {
        (*self).storage(address, index)
    }

    fn block_hash(&mut self, number: U256) -> H256 {
        (*self).block_hash(number)
    }

    fn account_mut(&mut self, address: H160) -> &mut AccountInfo {
        (*self).account_mut(address)
    }
}
