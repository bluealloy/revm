//! BlockHash database component from [`revm::Database`]

use auto_impl::auto_impl;
use core::ops::Deref;
use revm::primitives::B256;
use std::sync::Arc;

/// Trait for mutable access to block hash data.
/// This is typically used for database implementations that may cache or
/// lazily load block hashes.
#[auto_impl(&mut, Box)]
pub trait BlockHash {
    /// Error type for block hash operations
    type Error: core::error::Error + Send + Sync + 'static;

    /// Gets block hash by block number.
    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error>;
}

/// Trait for immutable access to block hash data.
/// This is typically used for read-only database implementations or
/// when block hash data is pre-loaded.
#[auto_impl(&, &mut, Box, Rc, Arc)]
pub trait BlockHashRef {
    /// Error type for block hash operations
    type Error: core::error::Error + Send + Sync + 'static;

    /// Gets block hash by block number.
    fn block_hash(&self, number: u64) -> Result<B256, Self::Error>;
}

impl<T> BlockHash for &T
where
    T: BlockHashRef,
{
    type Error = <T as BlockHashRef>::Error;

    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        BlockHashRef::block_hash(*self, number)
    }
}

impl<T> BlockHash for Arc<T>
where
    T: BlockHashRef,
{
    type Error = <T as BlockHashRef>::Error;

    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        self.deref().block_hash(number)
    }
}
