//! EIP-8037: State Creation Gas Cost Increase
//!
//! Introduces a reservoir model that separates *state gas* (storage/code/account
//! creation) from *regular* execution gas. State-gas charges are expressed as
//! a number of "state bytes" that get multiplied by `cost_per_state_byte` (CPSB).
//! In `bal-devnet-7` / Glamsterdam, CPSB is fixed at `1530`.

/// Blocks per year at a 12-second block time (used by the CPSB formula).
pub const BLOCKS_PER_YEAR: u64 = 2_628_000;

/// Target yearly state growth budget, in bytes.
pub const TARGET_STATE_GROWTH_PER_YEAR: u64 = 100 * 1024 * 1024 * 1024;

/// Offset subtracted after rounding in the CPSB formula.
pub const CPSB_OFFSET: u64 = 9578;

/// Number of high-order bits retained when rounding CPSB.
pub const CPSB_SIGNIFICANT_BITS: u32 = 5;

/// State bytes charged per SSTORE 0→non-zero.
pub const SSTORE_SET_BYTES: u64 = 64;

/// State bytes charged when creating a new account.
pub const NEW_ACCOUNT_BYTES: u64 = 120;

/// State bytes charged per EIP-7702 authorization base cost.
pub const AUTH_BASE_BYTES: u64 = 23;

/// State bytes charged per byte of deployed code.
pub const CODE_DEPOSIT_PER_BYTE: u64 = 1;

/// Regular gas component of EIP-7702 `PER_EMPTY_ACCOUNT_COST` under EIP-8037.
pub const EIP7702_PER_EMPTY_ACCOUNT_REGULAR: u64 = 7500;

/// Cost per state byte (CPSB) for Glamsterdam.
///
/// Reference: [EIP-8037: State Creation Gas Cost Increase](https://eips.ethereum.org/EIPS/eip-8037).
pub const CPSB_GLAMSTERDAM: u64 = 1530;
