//! State creation counters (EIP-8037).
//!
//! Tracks net new state created within a call frame (or across the whole
//! transaction at the top level): new storage slots, new accounts, and bytes
//! deposited as code on successful contract creations. Total state gas spent
//! is derivable from these counts via [`NewStateTracker::state_gas_spent`].

use crate::cfg::{GasId, GasParams};

/// Net counts of new state created in this scope.
///
/// Under EIP-8037 every state-creating operation charges
/// `unit_count × bytes_per_unit × cpsb` of state gas. By tracking the unit
/// counts here we can derive the total state gas spent at any point and
/// propagate state-gas accounting cleanly across parent/child frames.
///
/// Account creation is split into two counters because EIP-8037 distinguishes
/// the gas costs:
/// * `new_create_accounts` — paid via `create_state_gas` (CREATE / CREATE2).
/// * `new_call_accounts` — paid via `new_account_state_gas` (CALL with value
///   to an empty account, SELFDESTRUCT to an empty account).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NewStateTracker {
    /// Net number of new storage slots created (`SSTORE 0 → x`).
    ///
    /// Can be negative within a sub-frame when a `0 → x → 0` storage
    /// restoration removes a slot whose original `0 → x` charge happened on a
    /// parent frame. The parent's matching positive contribution is reconciled
    /// when child counters are merged into the parent on success.
    pub new_storages: i64,
    /// Net number of new accounts created via CREATE / CREATE2.
    pub new_create_accounts: u64,
    /// Net number of new accounts materialized via CALL with value to empty
    /// or SELFDESTRUCT to empty.
    pub new_call_accounts: u64,
    /// Net code-deposit bytes from successful contract creations.
    pub code_deposit_bytes: u64,
    /// Additional state gas spending. If it does not fit in any of above.
    pub additional_state_gas_spending: u64,
}

impl NewStateTracker {
    /// Creates an empty tracker.
    #[inline]
    pub const fn new() -> Self {
        Self {
            new_storages: 0,
            new_create_accounts: 0,
            new_call_accounts: 0,
            code_deposit_bytes: 0,
            additional_state_gas_spending: 0,
        }
    }

    /// Records a new storage slot creation (`SSTORE 0 → x`).
    #[inline]
    pub const fn add_storage(&mut self) {
        self.new_storages = self.new_storages.saturating_add(1);
    }

    /// Records a storage slot restoration (`SSTORE x → 0` against a slot whose
    /// original value at the start of the transaction was zero).
    #[inline]
    pub const fn remove_storage(&mut self) {
        self.new_storages = self.new_storages.saturating_sub(1);
    }

    /// Records a CREATE / CREATE2 new account.
    #[inline]
    pub const fn add_create_account(&mut self) {
        self.new_create_accounts = self.new_create_accounts.saturating_add(1);
    }

    /// Removes a previously-counted CREATE account (e.g. CREATE failure refund).
    #[inline]
    pub const fn remove_create_account(&mut self) {
        self.new_create_accounts = self.new_create_accounts.saturating_sub(1);
    }

    /// Records a CALL-with-value-to-empty / SELFDESTRUCT-to-empty new account.
    #[inline]
    pub const fn add_call_account(&mut self) {
        self.new_call_accounts = self.new_call_accounts.saturating_add(1);
    }

    /// Removes a previously-counted call-path account.
    #[inline]
    pub const fn remove_call_account(&mut self) {
        self.new_call_accounts = self.new_call_accounts.saturating_sub(1);
    }

    /// Records `bytes` deposited as code on contract creation success.
    #[inline]
    pub const fn add_code_deposit_bytes(&mut self, bytes: u64) {
        self.code_deposit_bytes = self.code_deposit_bytes.saturating_add(bytes);
    }

    /// Removes `bytes` of code deposit (e.g. selfdestruct refund of a
    /// contract that was created and destroyed within this transaction).
    #[inline]
    pub const fn remove_code_deposit_bytes(&mut self, bytes: u64) {
        self.code_deposit_bytes = self.code_deposit_bytes.saturating_sub(bytes);
    }

    /// Merges another tracker's deltas into this one. Used when a successful
    /// child frame's counters are absorbed by the parent.
    #[inline]
    pub const fn merge(&mut self, other: &NewStateTracker) {
        self.new_storages = self.new_storages.saturating_add(other.new_storages);
        self.new_create_accounts = self
            .new_create_accounts
            .saturating_add(other.new_create_accounts);
        self.new_call_accounts = self
            .new_call_accounts
            .saturating_add(other.new_call_accounts);
        self.code_deposit_bytes = self
            .code_deposit_bytes
            .saturating_add(other.code_deposit_bytes);
    }

    /// Resets all counters to zero.
    #[inline]
    pub const fn clear(&mut self) {
        self.new_storages = 0;
        self.new_create_accounts = 0;
        self.new_call_accounts = 0;
        self.code_deposit_bytes = 0;
    }

    /// Total state gas implied by these deltas given the gas table and CPSB.
    ///
    /// The result can be negative when `new_storages` is, signalling that a
    /// sub-frame uncreated more storage than it created (the missing positive
    /// contribution lives on a parent frame and is reconciled when counters
    /// are merged).
    #[inline]
    pub fn state_gas_spent(&self, gas_params: &GasParams, cpsb: u64) -> i64 {
        let storage_per = gas_params
            .get(GasId::sstore_set_state_gas())
            .saturating_mul(cpsb) as i64;
        let create_per = gas_params
            .get(GasId::create_state_gas())
            .saturating_mul(cpsb) as i64;
        let call_per = gas_params
            .get(GasId::new_account_state_gas())
            .saturating_mul(cpsb) as i64;
        let code_per_byte = gas_params
            .get(GasId::code_deposit_state_gas())
            .saturating_mul(cpsb) as i64;

        let storages = self.new_storages.saturating_mul(storage_per);
        let creates = (self.new_create_accounts as i64).saturating_mul(create_per);
        let calls = (self.new_call_accounts as i64).saturating_mul(call_per);
        let code = (self.code_deposit_bytes as i64).saturating_mul(code_per_byte);

        storages
            .saturating_add(creates)
            .saturating_add(calls)
            .saturating_add(code)
    }
}
