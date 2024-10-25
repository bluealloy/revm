//! BlockHash database component from [`revm::Database`]

use auto_impl::auto_impl;
use core::ops::Deref;
use revm::primitives::B256;
use std::sync::Arc;

#[auto_impl(&mut, Box)]
pub trait BlockHash {
    type Error: core::error::Error + 'static;

    /// Get block hash by block number
    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error>;
}

#[auto_impl(&, &mut, Box, Rc, Arc)]
pub trait BlockHashRef {
    type Error: core::error::Error + 'static;

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

/// Wraps a [`BlockHashRef`] to provide a [`BlockHash`] implementation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WrapBlockHashRef<T: BlockHashRef>(pub T);

impl<F: BlockHashRef> From<F> for WrapBlockHashRef<F> {
    #[inline]
    fn from(f: F) -> Self {
        WrapBlockHashRef(f)
    }
}

impl<T: BlockHashRef> BlockHash for WrapBlockHashRef<T> {
    type Error = T::Error;

    #[inline]
    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        self.0.block_hash(number)
    }
}
