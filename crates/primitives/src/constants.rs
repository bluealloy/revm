//! Global constants for the EVM
//!
//! Here you can find constants that dont belong to any EIP and are there for the genesis.

use crate::eip170;
use alloy_primitives::{b256, Address, B256};

/// Number of block hashes that EVM can access in the past (pre-Prague)
pub const BLOCK_HASH_HISTORY: u64 = 256;

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
    b256!("0xc5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470");
