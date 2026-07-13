//! EIP-8038: State-Access Gas Cost Update
//!
//! Increases the gas cost of state-access operations to reflect Ethereum's
//! larger state. The values below are the parameters proposed in
//! [ethereum/EIPs#11802](https://github.com/ethereum/EIPs/pull/11802) (still a
//! draft — treat as preliminary), superseding the earlier `previous_value + 1`
//! placeholders.
//!
//! Active alongside EIP-7904 and EIP-8037 starting at the Amsterdam hardfork.

/// Cold touch of an account (was 2,600 pre-EIP-8038).
pub const COLD_ACCOUNT_ACCESS: u64 = 3_000;

/// Surcharge for writing to an account that changes one account leaf value for
/// the first time (was 6,700 pre-EIP-8038).
pub const ACCOUNT_WRITE: u64 = 8_000;

/// Cold touch of a storage slot (was 2,100 pre-EIP-8038).
pub const COLD_STORAGE_ACCESS: u64 = 3_000;

/// Surcharge for writing to a storage slot that changes its value for the
/// first time (was 2,800 pre-EIP-8038).
pub const STORAGE_WRITE: u64 = 10_000;

/// Touch of an already-warm account or storage slot. Unchanged by EIP-8038.
pub const WARM_ACCESS: u64 = 100;

/// Refund for clearing a storage slot (was 4,800 pre-EIP-8038).
///
/// Derived per the spec as `(STORAGE_WRITE + COLD_STORAGE_ACCESS) * 4800 / 5000`.
pub const STORAGE_CLEAR_REFUND: u64 = (STORAGE_WRITE + COLD_STORAGE_ACCESS) * 4_800 / 5_000;

/// State access cost for contract deployment (was 7,000 pre-EIP-8038).
///
/// Per the spec, `CREATE_ACCESS = ACCOUNT_WRITE + COLD_STORAGE_ACCESS`. This does
/// not match the legacy decomposition (`GAS_CREATE - GAS_NEW_ACCOUNT = 7,000`); the
/// EIP keeps that discrepancy rather than reconciling it.
pub const CREATE_ACCESS: u64 = ACCOUNT_WRITE + COLD_STORAGE_ACCESS;

/// Gas charged per storage key included in a transaction's access list
/// (was 1,900 pre-EIP-8038). Derived per the spec as `COLD_STORAGE_ACCESS`.
pub const ACCESS_LIST_STORAGE_KEY_COST: u64 = COLD_STORAGE_ACCESS;

/// Gas charged per address included in a transaction's access list
/// (was 2,400 pre-EIP-8038). Derived per the spec as `COLD_ACCOUNT_ACCESS`.
pub const ACCESS_LIST_ADDRESS_COST: u64 = COLD_ACCOUNT_ACCESS;

/// Cold premium on top of `WARM_ACCESS` for account access.
pub const COLD_ACCOUNT_ACCESS_ADDITIONAL: u64 = COLD_ACCOUNT_ACCESS - WARM_ACCESS;

/// Cold premium on top of `WARM_ACCESS` for storage access.
pub const COLD_STORAGE_ACCESS_ADDITIONAL: u64 = COLD_STORAGE_ACCESS - WARM_ACCESS;

/// CALL value transfer cost: `ACCOUNT_WRITE + CALL_STIPEND` per the EIP.
pub const CALL_VALUE: u64 = ACCOUNT_WRITE + 2_300;

/// Calldata bytes charged for one EIP-7702 authorization tuple (execution-specs
/// `AUTH_TUPLE_BYTES`): chain id, authority address, nonce, signature parity, and
/// the two signature scalars. Charged at the calldata floor rate.
pub const EIP7702_AUTH_TUPLE_BYTES: u64 = 101;

/// ecRecover precompile base cost, charged once per EIP-7702 authorization to
/// recover the authority.
pub const EIP7702_ECRECOVER_COST: u64 = 3_000;

/// Calldata floor rate per token under EIP-7976 (Amsterdam).
pub const TX_DATA_TOKEN_FLOOR: u64 = 16;

/// `REGULAR_PER_AUTH_BASE_COST`: the state-independent regular-gas base charged
/// per EIP-7702 authorization at the intrinsic phase under EIP-2780
/// (ethereum/EIPs#11844).
///
/// `AUTH_TUPLE_BYTES * TX_DATA_TOKEN_FLOOR + PRECOMPILE_ECRECOVER + COLD_ACCOUNT_ACCESS + 2 * WARM_ACCESS`
/// = `101*16 + 3,000 + 3,000 + 200 = 7,816`. Covers the authorization's calldata,
/// the ECDSA recovery, the authority's cold access, and the warm writes every
/// authorization performs. The state-dependent remainder (`ACCOUNT_WRITE` and the
/// new-account / delegation-bytes state gas) is charged at runtime, only for the
/// authorities that incur it.
pub const EIP7702_PER_AUTH_BASE_REGULAR: u64 = EIP7702_AUTH_TUPLE_BYTES * TX_DATA_TOKEN_FLOOR
    + EIP7702_ECRECOVER_COST
    + COLD_ACCOUNT_ACCESS
    + 2 * WARM_ACCESS;

#[cfg(test)]
mod tests {
    use super::*;

    /// Values must match the parameters table in ethereum/EIPs#11802.
    #[test]
    fn constants_match_spec() {
        assert_eq!(WARM_ACCESS, 100); // unchanged by EIP-8038
        assert_eq!(COLD_ACCOUNT_ACCESS, 3_000);
        assert_eq!(ACCOUNT_WRITE, 8_000);
        assert_eq!(COLD_STORAGE_ACCESS, 3_000);
        assert_eq!(STORAGE_WRITE, 10_000);
        assert_eq!(STORAGE_CLEAR_REFUND, 12_480);
        assert_eq!(CREATE_ACCESS, 11_000);
        assert_eq!(ACCESS_LIST_ADDRESS_COST, 3_000);
        assert_eq!(ACCESS_LIST_STORAGE_KEY_COST, 3_000);
        assert_eq!(CALL_VALUE, 10_300);
        assert_eq!(EIP7702_PER_AUTH_BASE_REGULAR, 7_816);
    }

    /// Spec-defined relationships between the parameters (kept as derivations so a
    /// renumber of one base value propagates correctly).
    #[test]
    fn derived_relations() {
        assert_eq!(CREATE_ACCESS, ACCOUNT_WRITE + COLD_STORAGE_ACCESS);
        assert_eq!(CALL_VALUE, ACCOUNT_WRITE + 2_300);
        assert_eq!(ACCESS_LIST_ADDRESS_COST, COLD_ACCOUNT_ACCESS);
        assert_eq!(ACCESS_LIST_STORAGE_KEY_COST, COLD_STORAGE_ACCESS);
        assert_eq!(
            STORAGE_CLEAR_REFUND,
            (STORAGE_WRITE + COLD_STORAGE_ACCESS) * 4_800 / 5_000
        );
        assert_eq!(
            COLD_ACCOUNT_ACCESS_ADDITIONAL,
            COLD_ACCOUNT_ACCESS - WARM_ACCESS
        );
        assert_eq!(
            COLD_STORAGE_ACCESS_ADDITIONAL,
            COLD_STORAGE_ACCESS - WARM_ACCESS
        );
    }
}
