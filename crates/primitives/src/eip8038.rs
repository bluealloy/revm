//! EIP-8038: State-Access Gas Cost Update
//!
//! Increases the gas cost of state-access operations to reflect Ethereum's
//! larger state. All "TBD" values in the draft spec are filled in here as
//! `previous_value + 1`.
//!
//! Active alongside EIP-7904 and EIP-8037 starting at the Amsterdam hardfork.

/// Cold touch of an account (was 2,600 pre-EIP-8038).
pub const COLD_ACCOUNT_ACCESS: u64 = 2_601;

/// Surcharge for writing to an account that changes one account leaf value for
/// the first time (was 6,700 pre-EIP-8038).
pub const ACCOUNT_WRITE: u64 = 6_701;

/// Cold touch of a storage slot (was 2,100 pre-EIP-8038).
pub const COLD_STORAGE_ACCESS: u64 = 2_101;

/// Surcharge for writing to a storage slot that changes its value for the
/// first time (was 2,800 pre-EIP-8038).
pub const STORAGE_WRITE: u64 = 2_801;

/// Touch of an already-warm account or storage slot (was 100 pre-EIP-8038).
pub const WARM_ACCESS: u64 = 101;

/// Refund for clearing a storage slot (was 4,800 pre-EIP-8038).
pub const STORAGE_CLEAR_REFUND: u64 = 4_801;

/// State access cost for contract deployment (was 7,000 pre-EIP-8038).
///
/// Note: the spec also defines `CREATE_ACCESS = ACCOUNT_WRITE + COLD_STORAGE_ACCESS`,
/// which does not exactly match the legacy decomposition. Per the user-supplied
/// rule we keep this slot at the literal `previous + 1` value rather than the
/// derived sum.
pub const CREATE_ACCESS: u64 = 7_001;

/// Gas charged per storage key included in a transaction's access list
/// (was 1,900 pre-EIP-8038).
pub const ACCESS_LIST_STORAGE_KEY_COST: u64 = 1_901;

/// Gas charged per address included in a transaction's access list
/// (was 2,400 pre-EIP-8038).
pub const ACCESS_LIST_ADDRESS_COST: u64 = 2_401;

/// Cold premium on top of `WARM_ACCESS` for account access.
pub const COLD_ACCOUNT_ACCESS_ADDITIONAL: u64 = COLD_ACCOUNT_ACCESS - WARM_ACCESS;

/// Cold premium on top of `WARM_ACCESS` for storage access.
pub const COLD_STORAGE_ACCESS_ADDITIONAL: u64 = COLD_STORAGE_ACCESS - WARM_ACCESS;

/// CALL value transfer cost: `ACCOUNT_WRITE + CALL_STIPEND` per the EIP.
pub const CALL_VALUE: u64 = ACCOUNT_WRITE + 2_300;

/// Regular-gas portion of EIP-7702 `PER_EMPTY_ACCOUNT_COST` under EIP-8038.
///
/// Equal to the pre-EIP-8038 value (`crate::eip8037::EIP7702_PER_EMPTY_ACCOUNT_REGULAR`)
/// plus the delta from the modified primitives that appear in the per-auth
/// breakdown (`ACCOUNT_WRITE` +1, `COLD_ACCOUNT_ACCESS` +1, two `WARM_ACCESS`
/// occurrences +1 each — total +4).
pub const EIP7702_PER_EMPTY_ACCOUNT_REGULAR: u64 = 7_504;
