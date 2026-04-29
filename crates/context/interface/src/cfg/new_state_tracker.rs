//! State creation counters (EIP-8037).
//!
//! Tracks new state created within a call frame (or across the whole
//! transaction at the top level): new storage slots, new accounts, and bytes
//! deposited as code on successful contract creations. Total state gas
//! charged and refunded is derivable from these counts via
//! [`NewStateTracker::state_gas_charged_and_refunded`].

use crate::cfg::{GasId, GasParams};

/// Counts of new state created and unwound in this scope.
///
/// Under EIP-8037 every state-creating operation charges
/// `unit_count × bytes_per_unit × cpsb` of state gas. By tracking the unit
/// counts here we can derive the total state gas charged (and separately
/// refunded) at any point and propagate state-gas accounting cleanly across
/// parent/child frames.
///
/// Account creation is split into two counters because EIP-8037 distinguishes
/// the gas costs:
/// * `new_create_accounts` — paid via `create_state_gas` (CREATE / CREATE2).
/// * `new_call_accounts` — paid via `new_account_state_gas` (CALL with value
///   to an empty account, SELFDESTRUCT to an empty account).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NewStateTracker {
    /// Number of new storage slots created (`SSTORE 0 → x`).
    pub new_storages: u64,
    /// Number of storage slots restored (`SSTORE x → 0`) whose original
    /// pre-tx value was zero. Counted separately from `new_storages` so the
    /// charged and refunded contributions to state gas can be tracked
    /// independently.
    pub new_storages_refunded: u64,
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
            new_storages_refunded: 0,
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
        self.new_storages_refunded = self.new_storages_refunded.saturating_add(1);
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
        self.new_storages_refunded = self
            .new_storages_refunded
            .saturating_add(other.new_storages_refunded);
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
        self.new_storages_refunded = 0;
        self.new_create_accounts = 0;
        self.new_call_accounts = 0;
        self.code_deposit_bytes = 0;
    }

    /// State gas charged and refunded implied by these counts given the gas
    /// table and CPSB. The first element is the total charge (new storages,
    /// creates, calls, code-deposit bytes, and additional state gas spending).
    /// The second element is the total refund derived from
    /// `new_storages_refunded`.
    #[inline]
    pub fn state_gas_charged_and_refunded(
        &self,
        gas_params: &GasParams,
        cpsb: u64,
    ) -> (u64, u64) {
        let storage_per = gas_params
            .get(GasId::sstore_set_state_gas())
            .saturating_mul(cpsb);
        let create_per = gas_params
            .get(GasId::create_state_gas())
            .saturating_mul(cpsb);
        let call_per = gas_params
            .get(GasId::new_account_state_gas())
            .saturating_mul(cpsb);
        let code_per_byte = gas_params
            .get(GasId::code_deposit_state_gas())
            .saturating_mul(cpsb);

        let storages = self.new_storages.saturating_mul(storage_per);
        let creates = self.new_create_accounts.saturating_mul(create_per);
        let calls = self.new_call_accounts.saturating_mul(call_per);
        let code = self.code_deposit_bytes.saturating_mul(code_per_byte);

        let charged = storages
            .saturating_add(creates)
            .saturating_add(calls)
            .saturating_add(code)
            .saturating_add(self.additional_state_gas_spending);

        let refunded = self.new_storages_refunded.saturating_mul(storage_per);

        (charged, refunded)
    }
}
