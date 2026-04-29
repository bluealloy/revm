//! Per-frame state-gas accumulator (EIP-8037).
//!
//! State-creating opcodes (SSTORE 0→x, CREATE/CREATE2, CALL with value to an
//! empty account, SELFDESTRUCT, code deposit) bump the counters here instead
//! of charging the reservoir directly. At frame return the totals are
//! reconciled with the gas tracker:
//!
//! * **ok**: `state_gas_refunded` is added back to the reservoir first, then
//!   `state_gas` is charged from it (spilling into regular gas if the
//!   reservoir is exhausted, which can OOG the call).
//! * **revert / halt**: the counters are dropped — state work didn't happen.
//!
//! Hardcoded gas amounts use the Glamsterdam EIP-8037 defaults
//! (`bytes_per_unit × CPSB_GLAMSTERDAM`). A future change should derive these
//! from the active `GasParams` and `cpsb` at frame init.

use primitives::eip8037::{
    CODE_DEPOSIT_PER_BYTE, CPSB_GLAMSTERDAM, NEW_ACCOUNT_BYTES, SSTORE_SET_BYTES,
};

/// State gas charged for SSTORE 0→x.
pub const SSTORE_SET_STATE_GAS: u64 = SSTORE_SET_BYTES * CPSB_GLAMSTERDAM;
/// State gas charged when a CALL/SELFDESTRUCT materializes a new empty account.
pub const NEW_ACCOUNT_STATE_GAS: u64 = NEW_ACCOUNT_BYTES * CPSB_GLAMSTERDAM;
/// State gas charged upfront on CREATE/CREATE2 for the new account + metadata.
pub const CREATE_STATE_GAS: u64 = NEW_ACCOUNT_BYTES * CPSB_GLAMSTERDAM;
/// State gas charged per byte of deployed code on a successful contract creation.
pub const CODE_DEPOSIT_STATE_GAS_PER_BYTE: u64 = CODE_DEPOSIT_PER_BYTE * CPSB_GLAMSTERDAM;

/// Accumulates state gas charged and refunded within a single call frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NewStateTracker {
    /// Cumulative state gas charged in this frame (SSTORE 0→x, account
    /// creation, code deposit, etc.).
    pub state_gas: u64,
    /// Cumulative state gas refunded in this frame (SSTORE x→0 restoration
    /// against an originally-zero slot).
    pub state_gas_refunded: u64,
}

impl NewStateTracker {
    /// Creates an empty tracker.
    #[inline]
    pub const fn new() -> Self {
        Self {
            state_gas: 0,
            state_gas_refunded: 0,
        }
    }

    /// Records SSTORE 0→x.
    #[inline]
    pub const fn add_storage(&mut self) {
        self.state_gas = self.state_gas.saturating_add(SSTORE_SET_STATE_GAS);
    }

    /// Records SSTORE x→0 against an originally-zero slot.
    #[inline]
    pub const fn remove_storage(&mut self) {
        self.state_gas_refunded = self.state_gas_refunded.saturating_add(SSTORE_SET_STATE_GAS);
    }

    /// Records CREATE / CREATE2 new account.
    #[inline]
    pub const fn add_create_account(&mut self) {
        self.state_gas = self.state_gas.saturating_add(CREATE_STATE_GAS);
    }

    /// Removes a previously-counted CREATE account (failed CREATE refund).
    #[inline]
    pub const fn remove_create_account(&mut self) {
        self.state_gas = self.state_gas.saturating_sub(CREATE_STATE_GAS);
    }

    /// Records CALL-with-value-to-empty / SELFDESTRUCT-to-empty.
    #[inline]
    pub const fn add_call_account(&mut self) {
        self.state_gas = self.state_gas.saturating_add(NEW_ACCOUNT_STATE_GAS);
    }

    /// Records `bytes` deposited as code on contract creation success.
    #[inline]
    pub const fn add_code_deposit_bytes(&mut self, bytes: u64) {
        self.state_gas = self
            .state_gas
            .saturating_add(bytes.saturating_mul(CODE_DEPOSIT_STATE_GAS_PER_BYTE));
    }

    /// Merges another tracker's counts into this one. Used when a successful
    /// child frame's counters are absorbed by the parent.
    #[inline]
    pub const fn merge(&mut self, other: &NewStateTracker) {
        self.state_gas = self.state_gas.saturating_add(other.state_gas);
        self.state_gas_refunded = self
            .state_gas_refunded
            .saturating_add(other.state_gas_refunded);
    }

    /// Resets both counters to zero.
    #[inline]
    pub const fn clear(&mut self) {
        self.state_gas = 0;
        self.state_gas_refunded = 0;
    }
}
