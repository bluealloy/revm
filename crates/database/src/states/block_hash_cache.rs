use primitives::{alloy_primitives::B256, BLOCK_HASH_HISTORY};
use std::boxed::Box;

const BLOCK_HASH_HISTORY_USIZE: usize = BLOCK_HASH_HISTORY as usize;

/// A fixed-size cache for the 256 most recent block hashes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockHashCache {
    /// A fixed-size array holding the block hashes.
    /// Since we only store the most recent 256 block hashes, this array has a length of 256.
    /// The reason we store block number alongside its hash is to handle the case where it wraps around,
    /// so we can verify the block number. Uses `Option<B256>` to distinguish between "not cached"
    /// and "cached with value" - this is important because block 0 is a valid block number.
    hashes: Box<[(u64, Option<B256>); BLOCK_HASH_HISTORY_USIZE]>,
}

impl Default for BlockHashCache {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockHashCache {
    /// Creates a new empty BlockHashCache of length [BLOCK_HASH_HISTORY].
    #[inline]
    pub fn new() -> Self {
        Self {
            hashes: Box::new([(0, None); BLOCK_HASH_HISTORY_USIZE]),
        }
    }

    /// Inserts a block hash for the given block number.
    #[inline]
    pub fn insert(&mut self, block_number: u64, block_hash: B256) {
        let index = (block_number % BLOCK_HASH_HISTORY) as usize;
        self.hashes[index] = (block_number, Some(block_hash));
    }

    /// Retrieves the block hash for the given block number, if it exists in the cache.
    #[inline]
    pub fn get(&self, block_number: u64) -> Option<B256> {
        let index = (block_number % BLOCK_HASH_HISTORY) as usize;
        let (stored_block_number, stored_hash) = self.hashes[index];
        if stored_block_number == block_number {
            stored_hash
        } else {
            None
        }
    }
}
