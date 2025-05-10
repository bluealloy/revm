//! Optimism-specific constants, types, and helpers.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

mod account_info;
mod types;
pub use bytecode;

pub use account_info::AccountInfo;
pub use bytecode::Bytecode;
pub use primitives;
pub use types::{EvmState, EvmStorage, TransientStorage};

use bitflags::bitflags;
use core::hash::Hash;
use primitives::hardfork::SpecId;
use primitives::{HashMap, StorageKey, StorageValue};

/// Account type used inside Journal to track changed to state.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Account {
    /// Balance, nonce, and code
    pub info: AccountInfo,
    /// Storage cache
    pub storage: EvmStorage,
    /// Account status flags
    pub status: AccountStatus,
}

impl Account {
    /// Creates new account and mark it as non existing.
    pub fn new_not_existing() -> Self {
        Self {
            info: AccountInfo::default(),
            storage: HashMap::default(),
            status: AccountStatus::LoadedAsNotExisting,
        }
    }

    /// Checks if account is empty and check if empty state before spurious dragon hardfork.
    #[inline]
    pub fn state_clear_aware_is_empty(&self, spec: SpecId) -> bool {
        if SpecId::is_enabled_in(spec, SpecId::SPURIOUS_DRAGON) {
            self.is_empty()
        } else {
            let loaded_not_existing = self.is_loaded_as_not_existing();
            let is_not_touched = !self.is_touched();
            loaded_not_existing && is_not_touched
        }
    }

    /// Marks the account as self destructed.
    pub fn mark_selfdestruct(&mut self) {
        self.status |= AccountStatus::SelfDestructed;
    }

    /// Unmarks the account as self destructed.
    pub fn unmark_selfdestruct(&mut self) {
        self.status -= AccountStatus::SelfDestructed;
    }

    /// Is account marked for self destruct.
    pub fn is_selfdestructed(&self) -> bool {
        self.status.contains(AccountStatus::SelfDestructed)
    }

    /// Marks the account as touched
    pub fn mark_touch(&mut self) {
        self.status |= AccountStatus::Touched;
    }

    /// Unmarks the touch flag.
    pub fn unmark_touch(&mut self) {
        self.status -= AccountStatus::Touched;
    }

    /// If account status is marked as touched.
    pub fn is_touched(&self) -> bool {
        self.status.contains(AccountStatus::Touched)
    }

    /// Marks the account as newly created.
    pub fn mark_created(&mut self) {
        self.status |= AccountStatus::Created;
    }

    /// Unmarks the created flag.
    pub fn unmark_created(&mut self) {
        self.status -= AccountStatus::Created;
    }

    /// Marks the account as cold.
    pub fn mark_cold(&mut self) {
        self.status |= AccountStatus::Cold;
    }

    /// Marks the account as warm and return true if it was previously cold.
    pub fn mark_warm(&mut self) -> bool {
        if self.status.contains(AccountStatus::Cold) {
            self.status -= AccountStatus::Cold;
            true
        } else {
            false
        }
    }

    /// Is account loaded as not existing from database.
    ///
    /// This is needed for pre spurious dragon hardforks where
    /// existing and empty were two separate states.
    pub fn is_loaded_as_not_existing(&self) -> bool {
        self.status.contains(AccountStatus::LoadedAsNotExisting)
    }

    /// Is account newly created in this transaction.
    pub fn is_created(&self) -> bool {
        self.status.contains(AccountStatus::Created)
    }

    /// Is account empty, check if nonce and balance are zero and code is empty.
    pub fn is_empty(&self) -> bool {
        self.info.is_empty()
    }

    /// Returns an iterator over the storage slots that have been changed.
    ///
    /// See also [EvmStorageSlot::is_changed].
    pub fn changed_storage_slots(&self) -> impl Iterator<Item = (&StorageKey, &EvmStorageSlot)> {
        self.storage.iter().filter(|(_, slot)| slot.is_changed())
    }

    /// Sets account info and returns self for method chaining.
    pub fn with_info(mut self, info: AccountInfo) -> Self {
        self.info = info;
        self
    }

    /// Populates storage from an iterator of storage slots and returns self for method chaining.
    pub fn with_storage<I>(mut self, storage_iter: I) -> Self
    where
        I: Iterator<Item = (StorageKey, EvmStorageSlot)>,
    {
        for (key, slot) in storage_iter {
            self.storage.insert(key, slot);
        }
        self
    }

    /// Marks the account as self destructed and returns self for method chaining.
    pub fn with_selfdestruct_mark(mut self) -> Self {
        self.mark_selfdestruct();
        self
    }

    /// Marks the account as touched and returns self for method chaining.
    pub fn with_touched_mark(mut self) -> Self {
        self.mark_touch();
        self
    }

    /// Marks the account as newly created and returns self for method chaining.
    pub fn with_created_mark(mut self) -> Self {
        self.mark_created();
        self
    }

    /// Marks the account as cold and returns self for method chaining.
    pub fn with_cold_mark(mut self) -> Self {
        self.mark_cold();
        self
    }

    /// Marks the account as warm (not cold) and returns self for method chaining.
    /// Also returns whether the account was previously cold.
    pub fn with_warm_mark(mut self) -> (Self, bool) {
        let was_cold = self.mark_warm();
        (self, was_cold)
    }

    /// Variant of with_warm_mark that doesn't return the previous state.
    pub fn with_warm(mut self) -> Self {
        self.mark_warm();
        self
    }
}

impl From<AccountInfo> for Account {
    fn from(info: AccountInfo) -> Self {
        Self {
            info,
            storage: HashMap::default(),
            status: AccountStatus::Loaded,
        }
    }
}

// The `bitflags!` macro generates `struct`s that manage a set of flags.
bitflags! {
    /// Account status flags. Generated by bitflags crate.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "serde", serde(transparent))]
    pub struct AccountStatus: u8 {
        /// When account is loaded but not touched or interacted with.
        /// This is the default state.
        const Loaded = 0b00000000;
        /// When account is newly created we will not access database
        /// to fetch storage values
        const Created = 0b00000001;
        /// If account is marked for self destruction.
        const SelfDestructed = 0b00000010;
        /// Only when account is marked as touched we will save it to database.
        const Touched = 0b00000100;
        /// used only for pre spurious dragon hardforks where existing and empty were two separate states.
        /// it became same state after EIP-161: State trie clearing
        const LoadedAsNotExisting = 0b0001000;
        /// used to mark account as cold
        const Cold = 0b0010000;
    }
}

impl Default for AccountStatus {
    fn default() -> Self {
        Self::Loaded
    }
}

/// This type keeps track of the current value of a storage slot.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EvmStorageSlot {
    /// Original value of the storage slot
    pub original_value: StorageValue,
    /// Present value of the storage slot
    pub present_value: StorageValue,
    /// Represents if the storage slot is cold
    pub is_cold: bool,
}

impl EvmStorageSlot {
    /// Creates a new _unchanged_ `EvmStorageSlot` for the given value.
    pub fn new(original: StorageValue) -> Self {
        Self {
            original_value: original,
            present_value: original,
            is_cold: false,
        }
    }

    /// Creates a new _changed_ `EvmStorageSlot`.
    pub fn new_changed(original_value: StorageValue, present_value: StorageValue) -> Self {
        Self {
            original_value,
            present_value,
            is_cold: false,
        }
    }
    /// Returns true if the present value differs from the original value.
    pub fn is_changed(&self) -> bool {
        self.original_value != self.present_value
    }

    /// Returns the original value of the storage slot.
    pub fn original_value(&self) -> StorageValue {
        self.original_value
    }

    /// Returns the current value of the storage slot.
    pub fn present_value(&self) -> StorageValue {
        self.present_value
    }

    /// Marks the storage slot as cold.
    pub fn mark_cold(&mut self) {
        self.is_cold = true;
    }

    /// Marks the storage slot as warm and returns a bool indicating if it was previously cold.
    pub fn mark_warm(&mut self) -> bool {
        core::mem::replace(&mut self.is_cold, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EvmStorageSlot;
    use primitives::{StorageKey, KECCAK_EMPTY, U256};

    #[test]
    fn account_is_empty_balance() {
        let mut account = Account::default();
        assert!(account.is_empty());

        account.info.balance = U256::from(1);
        assert!(!account.is_empty());

        account.info.balance = U256::ZERO;
        assert!(account.is_empty());
    }

    #[test]
    fn account_is_empty_nonce() {
        let mut account = Account::default();
        assert!(account.is_empty());

        account.info.nonce = 1;
        assert!(!account.is_empty());

        account.info.nonce = 0;
        assert!(account.is_empty());
    }

    #[test]
    fn account_is_empty_code_hash() {
        let mut account = Account::default();
        assert!(account.is_empty());

        account.info.code_hash = [1; 32].into();
        assert!(!account.is_empty());

        account.info.code_hash = [0; 32].into();
        assert!(account.is_empty());

        account.info.code_hash = KECCAK_EMPTY;
        assert!(account.is_empty());
    }

    #[test]
    fn account_state() {
        let mut account = Account::default();

        assert!(!account.is_touched());
        assert!(!account.is_selfdestructed());

        account.mark_touch();
        assert!(account.is_touched());
        assert!(!account.is_selfdestructed());

        account.mark_selfdestruct();
        assert!(account.is_touched());
        assert!(account.is_selfdestructed());

        account.unmark_selfdestruct();
        assert!(account.is_touched());
        assert!(!account.is_selfdestructed());
    }

    #[test]
    fn account_is_cold() {
        let mut account = Account::default();

        // Account is not cold by default
        assert!(!account.status.contains(crate::AccountStatus::Cold));

        // When marking warm account as warm again, it should return false
        assert!(!account.mark_warm());

        // Mark account as cold
        account.mark_cold();

        // Account is cold
        assert!(account.status.contains(crate::AccountStatus::Cold));

        // When marking cold account as warm, it should return true
        assert!(account.mark_warm());
    }

    #[test]
    fn test_account_with_info() {
        let info = AccountInfo::default();
        let account = Account::default().with_info(info.clone());

        assert_eq!(account.info, info);
        assert_eq!(account.storage, HashMap::default());
        assert_eq!(account.status, AccountStatus::Loaded);
    }

    #[test]
    fn test_account_with_storage() {
        let mut storage = HashMap::new();
        let key1 = StorageKey::from(1);
        let key2 = StorageKey::from(2);
        let slot1 = EvmStorageSlot::new(StorageValue::from(10));
        let slot2 = EvmStorageSlot::new(StorageValue::from(20));

        storage.insert(key1, slot1.clone());
        storage.insert(key2, slot2.clone());

        let account = Account::default().with_storage(storage.clone().into_iter());

        assert_eq!(account.storage.len(), 2);
        assert_eq!(account.storage.get(&key1), Some(&slot1));
        assert_eq!(account.storage.get(&key2), Some(&slot2));
    }

    #[test]
    fn test_account_with_selfdestruct_mark() {
        let account = Account::default().with_selfdestruct_mark();

        assert!(account.is_selfdestructed());
        assert!(!account.is_touched());
        assert!(!account.is_created());
    }

    #[test]
    fn test_account_with_touched_mark() {
        let account = Account::default().with_touched_mark();

        assert!(!account.is_selfdestructed());
        assert!(account.is_touched());
        assert!(!account.is_created());
    }

    #[test]
    fn test_account_with_created_mark() {
        let account = Account::default().with_created_mark();

        assert!(!account.is_selfdestructed());
        assert!(!account.is_touched());
        assert!(account.is_created());
    }

    #[test]
    fn test_account_with_cold_mark() {
        let account = Account::default().with_cold_mark();

        assert!(account.status.contains(AccountStatus::Cold));
    }

    #[test]
    fn test_account_with_warm_mark() {
        // Start with a cold account
        let cold_account = Account::default().with_cold_mark();
        assert!(cold_account.status.contains(AccountStatus::Cold));

        // Use with_warm_mark to warm it
        let (warm_account, was_cold) = cold_account.with_warm_mark();

        // Check that it's now warm and previously was cold
        assert!(!warm_account.status.contains(AccountStatus::Cold));
        assert!(was_cold);

        // Try with an already warm account
        let (still_warm_account, was_cold) = warm_account.with_warm_mark();
        assert!(!still_warm_account.status.contains(AccountStatus::Cold));
        assert!(!was_cold);
    }

    #[test]
    fn test_account_with_warm() {
        // Start with a cold account
        let cold_account = Account::default().with_cold_mark();
        assert!(cold_account.status.contains(AccountStatus::Cold));

        // Use with_warm to warm it
        let warm_account = cold_account.with_warm();

        // Check that it's now warm
        assert!(!warm_account.status.contains(AccountStatus::Cold));
    }

    #[test]
    fn test_account_builder_chaining() {
        let info = AccountInfo {
            nonce: 5,
            ..AccountInfo::default()
        };

        let slot_key = StorageKey::from(42);
        let slot_value = EvmStorageSlot::new(StorageValue::from(123));
        let mut storage = HashMap::new();
        storage.insert(slot_key, slot_value.clone());

        // Chain multiple builder methods together
        let account = Account::default()
            .with_info(info.clone())
            .with_storage(storage.into_iter())
            .with_created_mark()
            .with_touched_mark()
            .with_cold_mark()
            .with_warm();

        // Verify all modifications were applied
        assert_eq!(account.info, info);
        assert_eq!(account.storage.get(&slot_key), Some(&slot_value));
        assert!(account.is_created());
        assert!(account.is_touched());
        assert!(!account.status.contains(AccountStatus::Cold));
    }
}
