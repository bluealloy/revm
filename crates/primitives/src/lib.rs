//! # revm-primitives
//!
//! Core primitive types and constants for the Ethereum Virtual Machine (EVM) implementation.
//!
//! This crate provides:
//! - EVM constants and limits (gas, stack, code size)
//! - Ethereum hard fork management and version control
//! - EIP-specific constants and configuration values
//! - Cross-platform synchronization primitives
//! - Type aliases for common EVM concepts (storage keys/values)
//! - Re-exports of alloy primitive types for convenience
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc as std;

pub mod constants;
pub mod eip170;
pub mod eip3860;
pub mod eip4844;
pub mod eip7702;
pub mod eip7823;
pub mod eip7825;
pub mod eip7907;
pub mod eip7918;
pub mod hardfork;
mod once_lock;

pub use constants::*;
pub use once_lock::OnceLock;

// Reexport alloy primitives.

pub use alloy_primitives::map::{self, hash_map, hash_set, HashMap, HashSet};
pub use alloy_primitives::{
    self, address, b256, bytes, fixed_bytes, hex, hex_literal, keccak256, ruint, uint, Address,
    Bytes, FixedBytes, Log, LogData, TxKind, B256, I128, I256, U128, U256,
};

/// Type alias for EVM storage keys (256-bit unsigned integers).
/// Used to identify storage slots within smart contract storage.
pub type StorageKey = U256;

/// Type alias for EVM storage values (256-bit unsigned integers).
/// Used to store data values in smart contract storage slots.
pub type StorageValue = U256;

/// Optimize short address access.
pub const SHORT_ADDRESS_CAP: usize = 300;

/// Returns the short address from Address.
///
/// Short address is considered address that has 18 leading zeros
/// and last two bytes are less than [`SHORT_ADDRESS_CAP`].
#[inline]
pub fn short_address(address: &Address) -> Option<usize> {
    if address.0[..18].iter().all(|b| *b == 0) {
        let short_address = u16::from_be_bytes([address.0[18], address.0[19]]) as usize;
        if short_address < SHORT_ADDRESS_CAP {
            return Some(short_address);
        }
    }
    None
}

/// 1 ether = 10^18 wei
pub const ONE_ETHER: u128 = 1_000_000_000_000_000_000;
