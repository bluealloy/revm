use alloy_primitives::{address, Address};

/// EIP-170: Contract code size limit
///
/// By default the limit is `0x6000` (~25kb)
pub const MAX_CODE_SIZE: usize = 0x6000;

/// Number of block hashes that EVM can access in the past (pre-Prague).
pub const BLOCK_HASH_HISTORY: usize = 256;

/// EIP-2935: Serve historical block hashes from state
///
/// Number of block hashes the EVM can access in the past (Prague).
///
/// # Note
///
/// This is named `HISTORY_SERVE_WINDOW` in the EIP.
pub const BLOCKHASH_SERVE_WINDOW: usize = 8192;

/// EIP-2935: Serve historical block hashes from state
///
/// The address where historical blockhashes are available.
///
/// # Note
///
/// This is named `HISTORY_STORAGE_ADDRESS` in the EIP.
pub const BLOCKHASH_STORAGE_ADDRESS: Address = address!("25a219378dad9b3503c8268c9ca836a52427a4fb");

/// EIP-3860: Limit and meter initcode
///
/// Limit of maximum initcode size is `2 * MAX_CODE_SIZE`.
pub const MAX_INITCODE_SIZE: usize = 2 * MAX_CODE_SIZE;

/// The address of precompile 3, which is handled specially in a few places.
pub const PRECOMPILE3: Address =
    Address::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3]);

// === EIP-4844 constants ===

/// Gas consumption of a single data blob (== blob byte size).
pub const GAS_PER_BLOB: u64 = 1 << 17;

/// Target number of the blob per block.
pub const TARGET_BLOB_NUMBER_PER_BLOCK: u64 = 3;

/// Max number of blobs per block
pub const MAX_BLOB_NUMBER_PER_BLOCK: u64 = 2 * TARGET_BLOB_NUMBER_PER_BLOCK;

/// Maximum consumable blob gas for data blobs per block.
pub const MAX_BLOB_GAS_PER_BLOCK: u64 = MAX_BLOB_NUMBER_PER_BLOCK * GAS_PER_BLOB;

/// Target consumable blob gas for data blobs per block (for 1559-like pricing).
pub const TARGET_BLOB_GAS_PER_BLOCK: u64 = TARGET_BLOB_NUMBER_PER_BLOCK * GAS_PER_BLOB;

/// Minimum gas price for data blobs.
pub const MIN_BLOB_GASPRICE: u64 = 1;

/// Controls the maximum rate of change for blob gas price.
pub const BLOB_GASPRICE_UPDATE_FRACTION: u64 = 3338477;

/// First version of the blob.
pub const VERSIONED_HASH_VERSION_KZG: u8 = 0x01;
