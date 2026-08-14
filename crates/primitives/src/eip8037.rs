//! EIP-8037: State Creation Gas Cost Increase
//!
//! Introduces a reservoir model that separates *state gas* (storage/code/account
//! creation) from *regular* execution gas. State-gas charges are expressed as
//! a number of "state bytes" that get multiplied by `cost_per_state_byte` (CPSB).
//! In Glamsterdam, CPSB is fixed at `1530`.

/// State bytes charged per SSTORE 0→non-zero.
pub const SSTORE_SET_BYTES: u64 = 64;

/// State bytes charged when creating a new account.
pub const NEW_ACCOUNT_BYTES: u64 = 120;

/// State bytes charged per EIP-7702 authorization base cost.
pub const AUTH_BASE_BYTES: u64 = 23;

/// State bytes charged per byte of deployed code.
pub const CODE_DEPOSIT_PER_BYTE: u64 = 1;

/// Cost per state byte (CPSB) for Glamsterdam.
///
/// Reference: [EIP-8037: State Creation Gas Cost Increase](https://eips.ethereum.org/EIPS/eip-8037).
pub const CPSB_GLAMSTERDAM: u64 = 1530;
