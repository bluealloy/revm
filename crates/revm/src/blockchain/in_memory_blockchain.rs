use crate::blockchain::{Blockchain, BlockchainRef};
use crate::common::keccak256;
use crate::{B256, U256};
use core::convert::Infallible;
use hashbrown::{hash_map::Entry, HashMap as Map};

pub type InMemoryBlockchain = CachedBlockchain<EmptyBlockchain>;

impl Default for InMemoryBlockchain {
    fn default() -> Self {
        CachedBlockchain::new(EmptyBlockchain)
    }
}

/// Memory backend, storing all state values in a `Map` in memory.
#[derive(Debug, Clone)]
pub struct CachedBlockchain<ExtBC: BlockchainRef> {
    pub block_hashes: Map<U256, B256>,
    pub blockchain: ExtBC,
}

impl<ExtBC: BlockchainRef> CachedBlockchain<ExtBC> {
    pub fn new(blockchain: ExtBC) -> Self {
        Self {
            block_hashes: Map::new(),
            blockchain,
        }
    }
}

impl<ExtBC: BlockchainRef> Blockchain for CachedBlockchain<ExtBC> {
    type Error = ExtBC::Error;

    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error> {
        match self.block_hashes.entry(number) {
            Entry::Occupied(entry) => Ok(*entry.get()),
            Entry::Vacant(entry) => {
                let hash = self.blockchain.block_hash(number)?;
                entry.insert(hash);
                Ok(hash)
            }
        }
    }
}

impl<ExtBC: BlockchainRef> BlockchainRef for CachedBlockchain<ExtBC> {
    type Error = ExtBC::Error;

    fn block_hash(&self, number: U256) -> Result<B256, Self::Error> {
        match self.block_hashes.get(&number) {
            Some(entry) => Ok(*entry),
            None => self.blockchain.block_hash(number),
        }
    }
}

/// An empty database that always returns default values when queried.
#[derive(Debug, Default, Clone)]
pub struct EmptyBlockchain;

impl Blockchain for EmptyBlockchain {
    type Error = Infallible;

    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error> {
        Ok(keccak256(&number.to_be_bytes::<{ U256::BYTES }>()))
    }
}

impl BlockchainRef for EmptyBlockchain {
    type Error = Infallible;

    fn block_hash(&self, number: U256) -> Result<B256, Self::Error> {
        Ok(keccak256(&number.to_be_bytes::<{ U256::BYTES }>()))
    }
}
