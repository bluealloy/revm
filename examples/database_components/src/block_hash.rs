//! BlockHash database component from [`revm::Database`]

use auto_impl::auto_impl;
use core::ops::Deref;
use revm::primitives::B256;
use std::sync::Arc;

#[auto_impl(&mut, Box)]
pub trait BlockHash {
    type Error;

    /// Get block hash by block number
    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error>;
}

#[auto_impl(&, &mut, Box, Rc, Arc)]
pub trait BlockHashRef {
    type Error;

    /// Get block hash by block number
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
