#[cfg(feature = "ethersdb")]
mod ethers_blockchain;
mod in_memory_blockchain;

#[cfg(feature = "ethersdb")]
pub use ethers_blockchain::EthersBlockchain;

pub use in_memory_blockchain::{CachedBlockchain, EmptyBlockchain, InMemoryBlockchain};

use auto_impl::auto_impl;

use crate::{B256, U256};

#[auto_impl(& mut, Box)]
pub trait Blockchain {
    type Error;

    // Get block hash by block number
    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error>;
}

#[auto_impl(&, Box, Arc)]
pub trait BlockchainRef {
    type Error;

    // Get block hash by block number
    fn block_hash(&self, number: U256) -> Result<B256, Self::Error>;
}

impl<T> Blockchain for &T
where
    T: BlockchainRef,
{
    type Error = <T as BlockchainRef>::Error;

    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error> {
        BlockchainRef::block_hash(*self, number)
    }
}
