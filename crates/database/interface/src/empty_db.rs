//! Empty database implementation.
use crate::{DBErrorMarker, Database, DatabaseRef};
use core::error::Error;
use core::{convert::Infallible, fmt, marker::PhantomData};
use primitives::{keccak256, Address, StorageKey, StorageValue, B256};
use state::{AccountInfo, Bytecode};
use std::string::ToString;

/// An empty database that always returns default values when queried
pub type EmptyDB = EmptyDBTyped<Infallible>;

/// An empty database that always returns default values when queried
///
/// This is generic over a type which is used as the database error type.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    /// Create a new empty database.
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<E: DBErrorMarker + Error> Database for EmptyDBTyped<E> {
    type Error = E;

    #[inline]
    async fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        <Self as DatabaseRef>::basic_ref(self, address).await
    }

    #[inline]
    async fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        <Self as DatabaseRef>::code_by_hash_ref(self, code_hash).await
    }

    #[inline]
    async fn storage(
        &mut self,
        address: Address,
        index: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        <Self as DatabaseRef>::storage_ref(self, address, index).await
    }

    #[inline]
    async fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        <Self as DatabaseRef>::block_hash_ref(self, number).await
    }
}

impl<E: DBErrorMarker + Error> DatabaseRef for EmptyDBTyped<E> {
    type Error = E;

    #[inline]
    async fn basic_ref(&self, _address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        Ok(None)
    }

    #[inline]
    async fn code_by_hash_ref(&self, _code_hash: B256) -> Result<Bytecode, Self::Error> {
        Ok(Bytecode::default())
    }

    #[inline]
    async fn storage_ref(
        &self,
        _address: Address,
        _index: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        Ok(StorageValue::default())
    }

    #[inline]
    async fn block_hash_ref(&self, number: u64) -> Result<B256, Self::Error> {
        Ok(keccak256(number.to_string().as_bytes()))
    }
}
