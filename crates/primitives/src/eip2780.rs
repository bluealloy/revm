//! EIP-2780: Reduce intrinsic transaction gas
//!
//! Replaces the legacy `21,000` intrinsic base with a decomposed model that
//! prices a reduced sender base plus additional `to`- and `value`-based
//! charges. Composes with EIP-8037 (state gas) and EIP-8038 (state-access
//! costs) starting at the Amsterdam hardfork.

/// Reduced intrinsic base cost charged to `tx.sender` (execution-specs `TX_BASE`).
pub const TX_BASE_COST: u64 = 12_000;

/// Regular gas cost of the EIP-7708 transfer log emitted for every nonzero-value
/// transfer to a different account.
///
/// `TRANSFER_LOG_COST = GAS_LOG + 3 * GAS_LOG_TOPIC + 32 * GAS_LOG_DATA_PER_BYTE`
/// = `375 + 1_125 + 256 = 1_756`.
pub const TRANSFER_LOG_COST: u64 = 1_756;

/// Additional intrinsic regular-gas charge for a value-bearing (non-create,
/// non-self) transaction (execution-specs `TX_VALUE_COST`), on top of
/// [`TRANSFER_LOG_COST`].
pub const TX_VALUE_COST: u64 = 4_244;
