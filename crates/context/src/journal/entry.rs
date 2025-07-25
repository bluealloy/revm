//! Contains the journal entry trait and implementations.
//!
//! Journal entries are used to track changes to the state and are used to revert it.
//!
//! They are created when there is change to the state from loading (making it warm), changes to the balance,
//! or removal of the storage slot. Check [`JournalEntryTr`] for more details.

use primitives::{Address, StorageKey, StorageValue, U256};
use state::{EvmState, TransientStorage};

/// Trait for tracking state changes in the EVM.
///
/// **IMPORTANT**: With the new snapshot-based checkpoint system, journal entries
/// are now mainly used for tracking and debugging purposes rather than for reverting state.
/// The complex revert logic has been removed in favor of simple state snapshots.
pub trait JournalEntryTr {
    /// Creates a journal entry for when an account is accessed and marked as "warm" for gas metering
    fn account_warmed(address: Address) -> Self;

    /// Creates a journal entry for when an account is destroyed via SELFDESTRUCT
    /// Records the target address that received the destroyed account's balance,
    /// whether the account was already destroyed, and its balance before destruction
    fn account_destroyed(
        address: Address,
        target: Address,
        destroyed_status: SelfdestructionRevertStatus,
        had_balance: U256,
    ) -> Self;

    /// Creates a journal entry for when an account is "touched" - accessed in a way that may require saving it.
    fn account_touched(address: Address) -> Self;

    /// Creates a journal entry for a balance transfer between accounts
    fn balance_transfer(from: Address, to: Address, balance: U256) -> Self;

    /// Creates a journal entry for when an account's balance is changed.
    fn balance_changed(address: Address, old_balance: U256) -> Self;

    /// Creates a journal entry for when an account's nonce is incremented.
    fn nonce_changed(address: Address) -> Self;

    /// Creates a journal entry for when a new account is created
    fn account_created(address: Address, is_created_globally: bool) -> Self;

    /// Creates a journal entry for when a storage slot is modified
    fn storage_changed(address: Address, key: StorageKey, had_value: StorageValue) -> Self;

    /// Creates a journal entry for when a storage slot is accessed and marked as "warm" for gas metering
    fn storage_warmed(address: Address, key: StorageKey) -> Self;

    /// Creates a journal entry for when a transient storage slot is modified (EIP-1153)
    fn transient_storage_changed(
        address: Address,
        key: StorageKey,
        had_value: StorageValue,
    ) -> Self;

    /// Creates a journal entry for when an account's code is modified
    fn code_changed(address: Address) -> Self;
}

/// Status of selfdestruction revert.
///
/// Global selfdestruction means that selfdestruct is called for first time in global scope.
///
/// Locally selfdesturction that selfdestruct is called for first time in one transaction scope.
///
/// Repeated selfdestruction means local selfdesturction was already called in one transaction scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SelfdestructionRevertStatus {
    /// Selfdestruct is called for first time in global scope.
    GloballySelfdestroyed,
    /// Selfdestruct is called for first time in one transaction scope.
    LocallySelfdestroyed,
    /// Selfdestruct is called again in one transaction scope.
    RepeatedSelfdestruction,
}

/// Journal entries that are used to track changes to the state and are used to revert it.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum JournalEntry {
    /// Used to mark account that is warm inside EVM in regard to EIP-2929 AccessList.
    /// Action: We will add Account to state.
    /// Revert: we will remove account from state.
    AccountWarmed {
        /// Address of warmed account.
        address: Address,
    },
    /// Mark account to be destroyed and journal balance to be reverted
    /// Action: Mark account and transfer the balance
    /// Revert: Unmark the account and transfer balance back
    AccountDestroyed {
        /// Balance of account got transferred to target.
        had_balance: U256,
        /// Address of account to be destroyed.
        address: Address,
        /// Address of account that received the balance.
        target: Address,
        /// Status of selfdestruction revert.
        destroyed_status: SelfdestructionRevertStatus,
    },
    /// Loading account does not mean that account will need to be added to MerkleTree (touched).
    /// Only when account is called (to execute contract or transfer balance) only then account is made touched.
    /// Action: Mark account touched
    /// Revert: Unmark account touched
    AccountTouched {
        /// Address of account that is touched.
        address: Address,
    },
    /// Balance changed
    /// Action: Balance changed
    /// Revert: Revert to previous balance
    BalanceChange {
        /// New balance of account.
        old_balance: U256,
        /// Address of account that had its balance changed.
        address: Address,
    },
    /// Transfer balance between two accounts
    /// Action: Transfer balance
    /// Revert: Transfer balance back
    BalanceTransfer {
        /// Balance that is transferred.
        balance: U256,
        /// Address of account that sent the balance.
        from: Address,
        /// Address of account that received the balance.
        to: Address,
    },
    /// Increment nonce
    /// Action: Increment nonce by one
    /// Revert: Decrement nonce by one
    NonceChange {
        /// Address of account that had its nonce changed.
        /// Nonce is incremented by one.
        address: Address,
    },
    /// Create account:
    /// Actions: Mark account as created
    /// Revert: Unmark account as created and reset nonce to zero.
    AccountCreated {
        /// Address of account that is created.
        /// On revert, this account will be set to empty.
        address: Address,
        /// If account is created globally for first time.
        is_created_globally: bool,
    },
    /// Entry used to track storage changes
    /// Action: Storage change
    /// Revert: Revert to previous value
    StorageChanged {
        /// Key of storage slot that is changed.
        key: StorageKey,
        /// Previous value of storage slot.
        had_value: StorageValue,
        /// Address of account that had its storage changed.
        address: Address,
    },
    /// Entry used to track storage warming introduced by EIP-2929.
    /// Action: Storage warmed
    /// Revert: Revert to cold state
    StorageWarmed {
        /// Key of storage slot that is warmed.
        key: StorageKey,
        /// Address of account that had its storage warmed. By SLOAD or SSTORE opcode.
        address: Address,
    },
    /// It is used to track an EIP-1153 transient storage change.
    /// Action: Transient storage changed.
    /// Revert: Revert to previous value.
    TransientStorageChange {
        /// Key of transient storage slot that is changed.
        key: StorageKey,
        /// Previous value of transient storage slot.
        had_value: StorageValue,
        /// Address of account that had its transient storage changed.
        address: Address,
    },
    /// Code changed
    /// Action: Account code changed
    /// Revert: Revert to previous bytecode.
    CodeChange {
        /// Address of account that had its code changed.
        address: Address,
    },
}
impl JournalEntryTr for JournalEntry {
    fn account_warmed(address: Address) -> Self {
        JournalEntry::AccountWarmed { address }
    }

    fn account_destroyed(
        address: Address,
        target: Address,
        destroyed_status: SelfdestructionRevertStatus,
        had_balance: U256,
    ) -> Self {
        JournalEntry::AccountDestroyed {
            address,
            target,
            destroyed_status,
            had_balance,
        }
    }

    fn account_touched(address: Address) -> Self {
        JournalEntry::AccountTouched { address }
    }

    fn balance_changed(address: Address, old_balance: U256) -> Self {
        JournalEntry::BalanceChange {
            address,
            old_balance,
        }
    }

    fn balance_transfer(from: Address, to: Address, balance: U256) -> Self {
        JournalEntry::BalanceTransfer { from, to, balance }
    }

    fn account_created(address: Address, is_created_globally: bool) -> Self {
        JournalEntry::AccountCreated {
            address,
            is_created_globally,
        }
    }

    fn storage_changed(address: Address, key: StorageKey, had_value: StorageValue) -> Self {
        JournalEntry::StorageChanged {
            address,
            key,
            had_value,
        }
    }

    fn nonce_changed(address: Address) -> Self {
        JournalEntry::NonceChange { address }
    }

    fn storage_warmed(address: Address, key: StorageKey) -> Self {
        JournalEntry::StorageWarmed { address, key }
    }

    fn transient_storage_changed(
        address: Address,
        key: StorageKey,
        had_value: StorageValue,
    ) -> Self {
        JournalEntry::TransientStorageChange {
            address,
            key,
            had_value,
        }
    }

    fn code_changed(address: Address) -> Self {
        JournalEntry::CodeChange { address }
    }
}
