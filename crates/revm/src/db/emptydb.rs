use core::{convert::Infallible, marker::PhantomData};
use revm_interpreter::primitives::{
    db::{Database, DatabaseRef},
    AccountInfo, Bytecode, B160, B256, U256,
};

/// An empty database that always returns default values when queried.
pub type EmptyDB = EmptyDBTyped<Infallible>;

/// An empty database that always returns default values when queried.
///
/// This is generic over a type which is used as the database error type.
#[derive(Debug, Clone)]
pub struct EmptyDBTyped<E> {
    _phantom: PhantomData<E>,
}

// Don't derive it because it doesn't need `E: Default`.
impl<E> Default for EmptyDBTyped<E> {
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl<E> EmptyDBTyped<E> {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    #[doc(hidden)]
    #[deprecated = "use `new` instead"]
    #[inline(always)]
    pub fn new_keccak_block_hash() -> Self {
        Self::new()
    }
}

impl<E> Database for EmptyDBTyped<E> {
    type Error = E;

    #[inline(always)]
    fn basic(&mut self, address: B160) -> Result<Option<AccountInfo>, Self::Error> {
        <Self as DatabaseRef>::basic(self, address)
    }

    #[inline(always)]
    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        <Self as DatabaseRef>::code_by_hash(self, code_hash)
    }

    #[inline(always)]
    fn storage(&mut self, address: B160, index: U256) -> Result<U256, Self::Error> {
        <Self as DatabaseRef>::storage(self, address, index)
    }

    #[inline(always)]
    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error> {
        <Self as DatabaseRef>::block_hash(self, number)
    }
}

impl<E> DatabaseRef for EmptyDBTyped<E> {
    type Error = E;

    #[inline(always)]
    fn basic(&self, _address: B160) -> Result<Option<AccountInfo>, Self::Error> {
        Ok(None)
    }

    #[inline(always)]
    fn code_by_hash(&self, _code_hash: B256) -> Result<Bytecode, Self::Error> {
        Ok(Bytecode::new())
    }

    #[inline(always)]
    fn storage(&self, _address: B160, _index: U256) -> Result<U256, Self::Error> {
        Ok(U256::default())
    }

    #[inline(always)]
    fn block_hash(&self, number: U256) -> Result<B256, Self::Error> {
        Ok(number.to_be_bytes().into())
    }
}
