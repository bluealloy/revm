//! EIP-7702: Set EOA Account Code
//!
//! Constants for account authorization and delegation functionality.

/// Base cost of updating authorized account.
pub const PER_AUTH_BASE_COST: u64 = 12500;

/// Cost of creating authorized account that was previously empty.
pub const PER_EMPTY_ACCOUNT_COST: u64 = 25000;
