use core::{convert::Infallible, fmt, marker::PhantomData};
use revm_interpreter::primitives::{
    db::{Database, DatabaseRef},
    AccountInfo, Address, Bytecode, B256, U256,
};

/// An empty database that always returns default values when queried.
pub type EmptyDB = EmptyDBTyped<Infallible>;

/// An empty database that always returns default values when queried.
///
/// This is generic over a type which is used as the database error type.
pub struct EmptyDBTyped<E> {
    _phantom: PhantomData<E>,
}

// Don't derive traits, because the type parameter is unused.
impl<E> Clone for EmptyDBTyped<E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<E> Copy for EmptyDBTyped<E> {}

impl<E> Default for EmptyDBTyped<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E> fmt::Debug for EmptyDBTyped<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EmptyDB").finish_non_exhaustive()
    }
}

impl<E> PartialEq for EmptyDBTyped<E> {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

impl<E> Eq for EmptyDBTyped<E> {}

impl<E> EmptyDBTyped<E> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }

    #[doc(hidden)]
    #[deprecated = "use `new` instead"]
    pub fn new_keccak_block_hash() -> Self {
        Self::new()
    }
}

impl<E> Database for EmptyDBTyped<E> {
    type Error = E;

    #[inline]
    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        <Self as DatabaseRef>::basic(self, address)
    }

    #[inline]
    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        <Self as DatabaseRef>::code_by_hash(self, code_hash)
    }

    #[inline]
    fn storage(&mut self, address: Address, index: U256) -> Result<U256, Self::Error> {
        <Self as DatabaseRef>::storage(self, address, index)
    }

    #[inline]
    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error> {
        <Self as DatabaseRef>::block_hash(self, number)
    }
}

impl<E> DatabaseRef for EmptyDBTyped<E> {
    type Error = E;

    #[inline]
    fn basic(&self, _address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        Ok(None)
    }

    #[inline]
    fn code_by_hash(&self, _code_hash: B256) -> Result<Bytecode, Self::Error> {
        Ok(Bytecode::new())
    }

    #[inline]
    fn storage(&self, _address: Address, _index: U256) -> Result<U256, Self::Error> {
        Ok(U256::default())
    }

    #[inline]
    fn block_hash(&self, number: U256) -> Result<B256, Self::Error> {
        Ok(number.to_be_bytes().into())
    }
}
