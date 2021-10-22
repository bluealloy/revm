use crate::{
    collection::{vec::Vec, Map},
    subroutine::Filth,
};

use primitive_types::{H160, H256, U256};

use super::trie;
use crate::{Account, AccountInfo, Log};
use bytes::Bytes;

pub trait Database {
    /// Whether account at address exists.
    fn exists(&mut self, address: H160) -> Option<AccountInfo>;
    /// Get basic account information.
    fn basic(&mut self, address: H160) -> AccountInfo;
    /// Get account code.
    fn code(&mut self, address: H160) -> Bytes;
    /// Get account code by its hash
    //fn code_by_hash(&mut self, code_hash: H256) -> Bytes;
    /// Get storage value of address at index.
    fn storage(&mut self, address: H160, index: H256) -> H256;

    // History related
    fn block_hash(&mut self, number: U256) -> H256;

    //apply
    //traces
}
