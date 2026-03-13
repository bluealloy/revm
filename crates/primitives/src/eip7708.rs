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

/// The topic hash for burn events.
///
/// This is emitted when a contract self-destructs to itself or when a
/// self-destructed account still has remaining balance at end of transaction.
/// `keccak256("Burn(address,uint256)")`
pub const BURN_LOG_TOPIC: B256 =
    b256!("0xcc16f5dbb4873280815c1ee09dbd06736cffcc184412cf7a71a0fdb75d397ca5");
