use primitives::{alloy_primitives::B256, BLOCK_HASH_HISTORY};

const BLOCK_HASH_HISTORY_USIZE: usize = BLOCK_HASH_HISTORY as usize;
const BLOCK_HASH_HISTORY_MINUS_ONE: u64 = BLOCK_HASH_HISTORY - 1;

/// A fixed-size cache for the 256 most recent block hashes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockHashCache {
    /// The block number corresponding to index 0 of the `hashes` array.
    start_block: u64,
    /// A fixed-size array holding the block hashes.
    /// Since we only store the most recent 256 block hashes, this array has a length of 256.
    /// The reason we store block number alongside its hash is to handle the case where it wraps around, so we can verify the block number.
    hashes: [(u64, B256); BLOCK_HASH_HISTORY_USIZE],
}

impl Default for BlockHashCache {
    fn default() -> Self {
        Self::new()
    }
}

impl BlockHashCache {
    /// Creates a new empty BlockHashCache.
    #[inline]
    pub const fn new() -> Self {
        Self {
            start_block: 0,
            hashes: [(0, B256::ZERO); BLOCK_HASH_HISTORY_USIZE],
        }
    }

    /// Inserts a block hash for the given block number.
    #[inline]
    pub const fn insert(&mut self, block_number: u64, block_hash: B256) {
        let index = (block_number % BLOCK_HASH_HISTORY) as usize;
        self.hashes[index] = (block_number, block_hash);
        if block_number >= self.start_block + BLOCK_HASH_HISTORY {
            // this only runs when block_number >= self.start_block + 256
            // Overflow impossible due to the check above
            self.start_block = block_number - BLOCK_HASH_HISTORY_MINUS_ONE;
        }
    }

    /// Retrieves the block hash for the given block number, if it exists in the cache.
    #[inline]
    pub const fn get(&self, block_number: u64) -> Option<B256> {
        let index = (block_number % BLOCK_HASH_HISTORY) as usize;
        let (stored_block_number, stored_hash) = self.hashes[index];
        if stored_block_number == block_number {
            Some(stored_hash)
        } else {
            None
        }
    }
}
