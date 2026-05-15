//! EIP-2780: Reduce intrinsic transaction gas
//!
//! Replaces the legacy `21,000` intrinsic base with a decomposed model that
//! prices ECDSA recovery, sender access + write, and additional `to`/`value`
//! charges separately. Composes with EIP-8037 (state gas) and reuses the
//! placeholder values from [`crate::eip8038`] for `ACCOUNT_WRITE`,
//! `CREATE_ACCESS`, and `COLD_ACCOUNT_ACCESS`.

/// Reduced intrinsic base cost charged to `tx.sender`.
///
/// Per the EIP, `TX_BASE_COST = GAS_PRECOMPILE_ECRECOVER + COLD_ACCOUNT_ACCESS + ACCOUNT_WRITE`.
/// Kept at the literal `12_300` from the EIP-2780 reference table; the `+1`
/// placeholder values in [`crate::eip8038`] would compute `12_302` instead.
pub const TX_BASE_COST: u64 = 12_300;

/// Regular gas cost of the EIP-7708 transfer log emitted for every nonzero-value
/// transfer to a different account.
///
/// `TRANSFER_LOG_COST = GAS_LOG + 3 * GAS_LOG_TOPIC + 32 * GAS_LOG_DATA_PER_BYTE`
/// = `375 + 1_125 + 256 = 1_756`.
pub const TRANSFER_LOG_COST: u64 = 1_756;
