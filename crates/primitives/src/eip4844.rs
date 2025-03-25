//! EIP-4844 constants
//!

/// First version of the blob
pub const VERSIONED_HASH_VERSION_KZG: u8 = 0x01;

/// Gas consumption of a single data blob (== blob byte size)
pub const GAS_PER_BLOB: u64 = 1 << 17;

/// Min blob gas price
pub const MIN_BLOB_GASPRICE: u64 = 1;

/// Target number of the blob per block
pub const TARGET_BLOB_NUMBER_PER_BLOCK_CANCUN: u64 = 3;

/// Max number of blobs per block
pub const MAX_BLOB_NUMBER_PER_BLOCK_CANCUN: u64 = 2 * TARGET_BLOB_NUMBER_PER_BLOCK_CANCUN;

/// Maximum consumable blob gas for data blobs per block
pub const MAX_BLOB_GAS_PER_BLOCK_CANCUN: u64 = MAX_BLOB_NUMBER_PER_BLOCK_CANCUN * GAS_PER_BLOB;

/// Target consumable blob gas for data blobs per block (for 1559-like pricing)
pub const TARGET_BLOB_GAS_PER_BLOCK_CANCUN: u64 =
    TARGET_BLOB_NUMBER_PER_BLOCK_CANCUN * GAS_PER_BLOB;

/// Controls the maximum rate of change for blob gas price
pub const BLOB_BASE_FEE_UPDATE_FRACTION_CANCUN: u64 = 3_338_477;

/// Target number of the blob per block
pub const TARGET_BLOB_NUMBER_PER_BLOCK_PRAGUE: u64 = 6;

/// Max number of blobs per block
pub const MAX_BLOB_NUMBER_PER_BLOCK_PRAGUE: u64 = 9;

/// Maximum consumable blob gas for data blobs per block
pub const MAX_BLOB_GAS_PER_BLOCK_PRAGUE: u64 = MAX_BLOB_NUMBER_PER_BLOCK_PRAGUE * GAS_PER_BLOB;

/// Target consumable blob gas for data blobs per block (for 1559-like pricing)
pub const TARGET_BLOB_GAS_PER_BLOCK_PRAGUE: u64 =
    TARGET_BLOB_NUMBER_PER_BLOCK_PRAGUE * GAS_PER_BLOB;

/// Controls the maximum rate of change for blob gas price
pub const BLOB_BASE_FEE_UPDATE_FRACTION_PRAGUE: u64 = 5_007_716;
