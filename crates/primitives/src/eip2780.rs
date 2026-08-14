//! EIP-2780: Reduce intrinsic transaction gas
//!
//! Replaces the legacy `21,000` intrinsic base with a decomposed model that
//! prices a reduced sender base plus additional `to`- and `value`-based
//! charges. Composes with EIP-8037 (state gas) and EIP-8038 (state-access
//! costs) starting at the Amsterdam hardfork.

/// Reduced intrinsic base cost charged to `tx.sender` (execution-specs `TX_BASE`).
pub const TX_BASE_COST: u64 = 12_000;

/// Additional intrinsic regular-gas charge for a value-bearing (non-create,
/// non-self) transaction (execution-specs `TX_VALUE_COST`).
///
/// Since glamsterdam devnet-8 the former separate `TRANSFER_LOG_COST` (1,756)
/// is folded into this constant (`4,244 + 1,756 = 6,000`); contract-creation
/// transactions no longer pay any value-based charge.
pub const TX_VALUE_COST: u64 = 6_000;
