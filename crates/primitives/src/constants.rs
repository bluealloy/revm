use alloy_primitives::{address, Address};

/// Number of block hashes that EVM can access in the past (pre-Prague).
pub const BLOCK_HASH_HISTORY: u64 = 256;

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

/// The address of precompile 3, which is handled specially in a few places.
pub const PRECOMPILE3: Address =
    Address::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3]);
