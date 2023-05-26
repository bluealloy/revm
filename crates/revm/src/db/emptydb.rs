use core::convert::Infallible;
use revm_interpreter::primitives::{
    db::{Database, DatabaseRef},
    keccak256, AccountInfo, Bytecode, B160, B256, U256,
};

/// An empty database that always returns default values when queried.
#[derive(Debug, Default, Clone)]
pub struct EmptyDB {
    pub keccak_block_hash: bool,
}

impl EmptyDB {
    pub fn new_keccak_block_hash() -> Self {
        Self {
            keccak_block_hash: true,
        }
    }
}

impl Database for EmptyDB {
    type Error = Infallible;

    fn basic(&mut self, address: B160) -> Result<Option<AccountInfo>, Self::Error> {
        <Self as DatabaseRef>::basic(self, address)
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        <Self as DatabaseRef>::code_by_hash(self, code_hash)
    }

    fn storage(&mut self, address: B160, index: U256) -> Result<U256, Self::Error> {
        <Self as DatabaseRef>::storage(self, address, index)
    }

    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error> {
        <Self as DatabaseRef>::block_hash(self, number)
    }
}

impl DatabaseRef for EmptyDB {
    type Error = Infallible;
    /// Get basic account information.
    fn basic(&self, _address: B160) -> Result<Option<AccountInfo>, Self::Error> {
        Ok(None)
    }
    /// Get account code by its hash
    fn code_by_hash(&self, _code_hash: B256) -> Result<Bytecode, Self::Error> {
        Ok(Bytecode::new())
    }
    /// Get storage value of address at index.
    fn storage(&self, _address: B160, _index: U256) -> Result<U256, Self::Error> {
        Ok(U256::default())
    }

    // History related
    fn block_hash(&self, number: U256) -> Result<B256, Self::Error> {
        Ok(keccak256(&number.to_be_bytes::<{ U256::BYTES }>()))
    }
}
