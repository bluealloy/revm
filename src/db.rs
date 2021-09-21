use primitive_types::{H160, H256, U256};

use crate::Basic;
use bytes::Bytes;

pub trait Database {
    /// Whether account at address exists.
    fn exists(&mut self, address: H160) -> bool;
    /// Get basic account information.
    fn basic(&mut self, address: H160) -> Basic;
    /// Get account code.
    fn code(&mut self, address: H160) -> Bytes;
    /// Get storage value of address at index.
    fn storage(&mut self, address: H160, index: H256) -> H256;
    /// Get original storage value of address at index, if available.
    fn original_storage(&mut self, address: H160, index: H256) -> Option<H256>;

    // History related
    fn block_hash(&mut self, number: U256) -> H256;

    //apply
    //traces
}


pub struct DummyDB;

impl Database for DummyDB {
    fn exists(&mut self, address: H160) -> bool {
        todo!()
    }

    fn basic(&mut self, address: H160) -> Basic {
        todo!()
    }

    fn code(&mut self, address: H160) -> Bytes {
        todo!()
    }

    fn storage(&mut self, address: H160, index: H256) -> H256 {
        todo!()
    }

    fn original_storage(&mut self, address: H160, index: H256) -> Option<H256> {
        todo!()
    }

    fn block_hash(&mut self, number: U256) -> H256 {
        todo!()
    }
}