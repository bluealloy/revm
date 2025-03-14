use crate::eip170;
use alloy_primitives::{b256, Address, B256};

/// Number of block hashes that EVM can access in the past (pre-Prague)
pub const BLOCK_HASH_HISTORY: u64 = 256;

/// EIP-2935: Serve historical block hashes from state
///
/// Number of block hashes the EVM can access in the past (Prague).
///
/// # Note
/// This is named `HISTORY_SERVE_WINDOW` in the EIP.
///
/// Updated from 8192 to 8191 in <https://github.com/ethereum/EIPs/pull/9144>
pub const BLOCKHASH_SERVE_WINDOW: usize = 8191;

/// The address of precompile 3, which is handled specially in a few places
pub const PRECOMPILE3: Address =
    Address::new([0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3]);

/// EVM interpreter stack limit
pub const STACK_LIMIT: usize = 1024;

/// EIP-3860: Limit and meter initcode
///
/// Limit of maximum initcode size is `2 * MAX_CODE_SIZE`.
pub const MAX_INITCODE_SIZE: usize = 2 * eip170::MAX_CODE_SIZE;

/// EVM call stack limit
pub const CALL_STACK_LIMIT: u64 = 1024;

/// The Keccak-256 hash of the empty string `""`.
pub const KECCAK_EMPTY: B256 =
    b256!("c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470");
