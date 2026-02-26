//! Constants for [EIP-7708](https://eips.ethereum.org/EIPS/eip-7708): ETH transfers emit a log.
//!
//! This EIP specifies that all ETH transfers (transactions, CALL, SELFDESTRUCT) emit a log,
//! making ETH transfers trackable like ERC-20 tokens.

use alloy_primitives::{address, b256, Address, B256};

/// The system address used as the log emitter for ETH transfer events.
///
/// This matches the ERC-20 Transfer event format but uses a system address
/// as the emitter since no contract actually emits these logs.
pub const ETH_TRANSFER_LOG_ADDRESS: Address =
    address!("0xfffffffffffffffffffffffffffffffffffffffe");

/// The topic hash for the Transfer event: `keccak256("Transfer(address,address,uint256)")`.
///
/// This is the same topic used by ERC-20 tokens for transfer events, ensuring
/// compatibility with existing indexing and tracking infrastructure.
pub const ETH_TRANSFER_LOG_TOPIC: B256 =
    b256!("0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef");

/// The topic hash for selfdestruct events (burn).
///
/// This is emitted when a contract self-destructs to itself or when a
/// self-destructed account still has remaining balance at end of transaction.
/// `keccak256("Selfdestruct(address,uint256)")`
pub const SELFDESTRUCT_LOG_TOPIC: B256 =
    b256!("0x4bfaba3443c1a1836cd362418edc679fc96cae8449cbefccb6457cdf2c943083");
