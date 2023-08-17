use core::{convert::Infallible, marker::PhantomData};
use revm_interpreter::primitives::{
    db::{Database, DatabaseRef},
    keccak256, AccountInfo, Address, Bytecode, B256, U256,
};

pub type EmptyDB = EmptyDBTyped<Infallible>;

impl Default for EmptyDB {
    fn default() -> Self {
        Self {
            keccak_block_hash: false,
            _phantom: PhantomData,
        }
    }
}

/// An empty database that always returns default values when queried.
#[derive(Debug, Clone)]
pub struct EmptyDBTyped<T> {
    pub keccak_block_hash: bool,
    pub _phantom: PhantomData<T>,
}

impl<T> EmptyDBTyped<T> {
    pub fn new() -> Self {
        Self {
            keccak_block_hash: false,
            _phantom: PhantomData,
        }
    }

    pub fn new_keccak_block_hash() -> Self {
        Self {
            keccak_block_hash: true,
            _phantom: PhantomData,
        }
    }
}

impl<T> Database for EmptyDBTyped<T> {
    type Error = T;

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        <Self as DatabaseRef>::basic(self, address)
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        <Self as DatabaseRef>::code_by_hash(self, code_hash)
    }

    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
        <Self as DatabaseRef>::storage(self, address, index)
    }

    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error> {
        <Self as DatabaseRef>::block_hash(self, number)
    }
}

impl<T> DatabaseRef for EmptyDBTyped<T> {
    type Error = T;
    /// Get basic account information.
    fn basic(&self, _address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        Ok(None)
    }
    /// Get account code by its hash
    fn code_by_hash(&self, _code_hash: B256) -> Result<Bytecode, Self::Error> {
        Ok(Bytecode::new())
    }
    /// Get storage value of address at index.
    fn storage(&self, _address: Address, _index: U256) -> Result<U256, Self::Error> {
        Ok(U256::default())
    }

    // History related
    fn block_hash(&self, number: U256) -> Result<B256, Self::Error> {
        Ok(keccak256(number.to_be_bytes::<{ U256::BYTES }>()))
    }
}
