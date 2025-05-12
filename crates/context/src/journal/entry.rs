//! Contains the journal entry trait and implementations.
//!
//! Journal entries are used to track changes to the state and are used to revert it.
//!
//! They are created when there is change to the state from loading (making it warm), changes to the balance,
//! or removal of the storage slot. Check [`JournalEntryTr`] for more details.
use primitives::{Address, StorageKey, StorageValue, KECCAK_EMPTY, PRECOMPILE3, U256};
use state::{EvmState, TransientStorage};

/// Trait for tracking and reverting state changes in the EVM.
/// Journal entry contains information about state changes that can be reverted.
pub trait JournalEntryTr {
    /// Creates a journal entry for when an account is accessed and marked as "warm" for gas metering
    fn account_warmed(address: Address) -> Self;

    /// Creates a journal entry for when an account is destroyed via SELFDESTRUCT
    /// Records the target address that received the destroyed account's balance,
    /// whether the account was already destroyed, and its balance before destruction
    /// on revert, the balance is transferred back to the original account
    fn account_destroyed(
        address: Address,
        target: Address,
        was_destroyed: bool,
        had_balance: U256,
    ) -> Self;

    /// Creates a journal entry for when an account is "touched" - accessed in a way that may require saving it.
    /// If account is empty and touch it will be removed from the state (EIP-161 state clear EIP)
    fn account_touched(address: Address) -> Self;

    /// Creates a journal entry for a balance transfer between accounts
    fn balance_transfer(from: Address, to: Address, balance: U256) -> Self;

    /// Creates a journal entry for when an account's nonce is incremented.
    fn nonce_changed(address: Address) -> Self;

    /// Creates a journal entry for when a new account is created
    fn account_created(address: Address) -> Self;

    /// Creates a journal entry for when a storage slot is modified
    /// Records the previous value for reverting
    fn storage_changed(address: Address, key: StorageKey, had_value: StorageValue) -> Self;

    /// Creates a journal entry for when a storage slot is accessed and marked as "warm" for gas metering
    /// This is called with SLOAD opcode.
    fn storage_warmed(address: Address, key: StorageKey) -> Self;

    /// Creates a journal entry for when a transient storage slot is modified (EIP-1153)
    /// Records the previous value for reverting
    fn transient_storage_changed(
        address: Address,
        key: StorageKey,
        had_value: StorageValue,
    ) -> Self;

    /// Creates a journal entry for when an account's code is modified
    fn code_changed(address: Address) -> Self;

    /// Reverts the state change recorded by this journal entry
    ///
    /// More information on what is reverted can be found in [`JournalEntry`] enum.
    ///
    /// # Notes
    ///   
    /// The spurious dragon flag is used to skip revertion 0x000..0003 precompile. This
    /// Behaviour is special and it caused by bug in Geth and Parity that is explained in [PR#716](https://github.com/ethereum/EIPs/issues/716).
    ///
    /// From yellow paper:
    /// ```text
    /// K.1. Deletion of an Account Despite Out-of-gas. At block 2675119, in the transaction 0xcf416c536ec1a19ed1fb89e
    /// 4ec7ffb3cf73aa413b3aa9b77d60e4fd81a4296ba, an account at address 0x03 was called and an out-of-gas occurred during
    /// the call. Against the equation (209), this added 0x03 in the set of touched addresses, and this transaction turned σ[0x03]
    /// into ∅.
    /// ```
    fn revert(
        self,
        state: &mut EvmState,
        transient_storage: &mut TransientStorage,
        is_spurious_dragon_enabled: bool,
    );
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
        /// Address of account to be destroyed.
        address: Address,
        /// Address of account that received the balance.
        target: Address,
        /// Whether the account had already been destroyed before this journal entry.
        was_destroyed: bool,
        /// Balance of account got transferred to target.
        had_balance: U256,
    },
    /// Loading account does not mean that account will need to be added to MerkleTree (touched).
    /// Only when account is called (to execute contract or transfer balance) only then account is made touched.
    /// Action: Mark account touched
    /// Revert: Unmark account touched
    AccountTouched {
        /// Address of account that is touched.
        address: Address,
    },
    /// Transfer balance between two accounts
    /// Action: Transfer balance
    /// Revert: Transfer balance back
    BalanceTransfer {
        /// Address of account that sent the balance.
        from: Address,
        /// Address of account that received the balance.
        to: Address,
        /// Balance that is transferred.
        balance: U256,
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
    },
    /// Entry used to track storage changes
    /// Action: Storage change
    /// Revert: Revert to previous value
    StorageChanged {
        /// Address of account that had its storage changed.
        address: Address,
        /// Key of storage slot that is changed.
        key: StorageKey,
        /// Previous value of storage slot.
        had_value: StorageValue,
    },
    /// Entry used to track storage warming introduced by EIP-2929.
    /// Action: Storage warmed
    /// Revert: Revert to cold state
    StorageWarmed {
        /// Address of account that had its storage warmed. By SLOAD or SSTORE opcode.
        address: Address,
        /// Key of storage slot that is warmed.
        key: StorageKey,
    },
    /// It is used to track an EIP-1153 transient storage change.
    /// Action: Transient storage changed.
    /// Revert: Revert to previous value.
    TransientStorageChange {
        /// Address of account that had its transient storage changed.
        address: Address,
        /// Key of transient storage slot that is changed.
        key: StorageKey,
        /// Previous value of transient storage slot.
        had_value: StorageValue,
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
        was_destroyed: bool, // if account had already been destroyed before this journal entry
        had_balance: U256,
    ) -> Self {
        JournalEntry::AccountDestroyed {
            address,
            target,
            was_destroyed,
            had_balance,
        }
    }

    fn account_touched(address: Address) -> Self {
        JournalEntry::AccountTouched { address }
    }

    fn balance_transfer(from: Address, to: Address, balance: U256) -> Self {
        JournalEntry::BalanceTransfer { from, to, balance }
    }

    fn account_created(address: Address) -> Self {
        JournalEntry::AccountCreated { address }
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

    fn revert(
        self,
        state: &mut EvmState,
        transient_storage: &mut TransientStorage,
        is_spurious_dragon_enabled: bool,
    ) {
        match self {
            JournalEntry::AccountWarmed { address } => {
                state.get_mut(&address).unwrap().mark_cold();
            }
            JournalEntry::AccountTouched { address } => {
                if is_spurious_dragon_enabled && address == PRECOMPILE3 {
                    return;
                }
                // remove touched status
                state.get_mut(&address).unwrap().unmark_touch();
            }
            JournalEntry::AccountDestroyed {
                address,
                target,
                was_destroyed,
                had_balance,
            } => {
                let account = state.get_mut(&address).unwrap();
                // set previous state of selfdestructed flag, as there could be multiple
                // selfdestructs in one transaction.
                if was_destroyed {
                    // flag is still selfdestructed
                    account.mark_selfdestruct();
                } else {
                    // flag that is not selfdestructed
                    account.unmark_selfdestruct();
                }
                account.info.balance += had_balance;

                if address != target {
                    let target = state.get_mut(&target).unwrap();
                    target.info.balance -= had_balance;
                }
            }
            JournalEntry::BalanceTransfer { from, to, balance } => {
                // we don't need to check overflow and underflow when adding and subtracting the balance.
                let from = state.get_mut(&from).unwrap();
                from.info.balance += balance;
                let to = state.get_mut(&to).unwrap();
                to.info.balance -= balance;
            }
            JournalEntry::NonceChange { address } => {
                state.get_mut(&address).unwrap().info.nonce -= 1;
            }
            JournalEntry::AccountCreated { address } => {
                let account = &mut state.get_mut(&address).unwrap();
                account.unmark_created();
                account.info.nonce = 0;
            }
            JournalEntry::StorageWarmed { address, key } => {
                state
                    .get_mut(&address)
                    .unwrap()
                    .storage
                    .get_mut(&key)
                    .unwrap()
                    .mark_cold();
            }
            JournalEntry::StorageChanged {
                address,
                key,
                had_value,
            } => {
                state
                    .get_mut(&address)
                    .unwrap()
                    .storage
                    .get_mut(&key)
                    .unwrap()
                    .present_value = had_value;
            }
            JournalEntry::TransientStorageChange {
                address,
                key,
                had_value,
            } => {
                let tkey = (address, key);
                if had_value.is_zero() {
                    // if previous value is zero, remove it
                    transient_storage.remove(&tkey);
                } else {
                    // if not zero, reinsert old value to transient storage.
                    transient_storage.insert(tkey, had_value);
                }
            }
            JournalEntry::CodeChange { address } => {
                let acc = state.get_mut(&address).unwrap();
                acc.info.code_hash = KECCAK_EMPTY;
                acc.info.code = None;
            }
        }
    }
}
