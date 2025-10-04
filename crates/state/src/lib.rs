//! Account and storage state.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

mod account_info;
pub mod bal;
mod types;
pub use bytecode;

pub use account_info::AccountInfo;
pub use bytecode::Bytecode;
pub use primitives;
pub use types::{EvmState, EvmStorage, TransientStorage};

use bitflags::bitflags;
use primitives::hardfork::SpecId;
use primitives::{HashMap, StorageKey, StorageValue, U256};

use crate::bal::account::AccountInfoBal;
use crate::bal::writes::BalWrites;
use crate::bal::AccountBal;

/// The main account type used inside Revm. It is stored inside Journal and contains all the information about the account.
///
/// Other than standard Account information it contains its status that can be both cold and warm
/// additional to that it contains BAL that is used to load data for this particular account.
///
/// On loading from database:
///     * If CompiledBal is present, load values from BAL into Account (Assume account has read data from database)
///     * In case of parallel execution, AccountInfo would be same over all parallel executions.
///     * Maybe use transaction_id as a way to notify user that this is obsolete data.
///     * Database needs to load account and tie to with BAL writes
/// If CompiledBal is not present, use loaded values
///     * Account is already up to date (uses present flow).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Account {
    /// Balance, nonce, and code
    pub info: AccountInfo,
    /// Original account info used by BAL, changed only on cold load by BAL.
    pub original_info: AccountInfo,
    /// Transaction id, used to track when account was toched/loaded into journal.
    pub transaction_id: usize,
    /// Storage cache
    pub storage: EvmStorage,
    /// Account status flags
    pub status: AccountStatus,
    /// BAL for account. Contains all writes values of the account info.
    ///
    /// If account is cold loaded, values of nonce/balance/code should be read from here.
    pub bal: AccountBal,
}

impl Account {
    /// Creates new account and mark it as non existing.
    pub fn new_not_existing(transaction_id: usize) -> Self {
        Self {
            info: AccountInfo::default(),
            storage: HashMap::default(),
            transaction_id,
            status: AccountStatus::LoadedAsNotExisting,
            original_info: AccountInfo::default(),
            bal: AccountBal::default(),
        }
    }

    /// Make changes to the caller account.
    ///
    /// It marks the account as touched, changes the balance and bumps the nonce if `is_call` is true.
    ///
    /// Returns the old balance.
    #[inline]
    pub fn caller_initial_modification(&mut self, new_balance: U256, is_call: bool) -> U256 {
        // Touch account so we know it is changed.
        self.mark_touch();

        if is_call {
            // Nonce is already checked
            self.info.nonce = self.info.nonce.saturating_add(1);
        }

        core::mem::replace(&mut self.info.balance, new_balance)
    }

    /// Checks if account is empty and check if empty state before spurious dragon hardfork.
    #[inline]
    pub fn state_clear_aware_is_empty(&self, spec: SpecId) -> bool {
        if SpecId::is_enabled_in(spec, SpecId::SPURIOUS_DRAGON) {
            self.is_empty()
        } else {
            self.is_loaded_as_not_existing_not_touched()
        }
    }

    /// Marks the account as self destructed.
    #[inline]
    pub fn mark_selfdestruct(&mut self) {
        self.status |= AccountStatus::SelfDestructed;
    }

    /// Unmarks the account as self destructed.
    #[inline]
    pub fn unmark_selfdestruct(&mut self) {
        self.status -= AccountStatus::SelfDestructed;
    }

    /// Is account marked for self destruct.
    #[inline]
    pub fn is_selfdestructed(&self) -> bool {
        self.status.contains(AccountStatus::SelfDestructed)
    }

    /// Marks the account as touched
    #[inline]
    pub fn mark_touch(&mut self) {
        self.status |= AccountStatus::Touched;
    }

    /// Unmarks the touch flag.
    #[inline]
    pub fn unmark_touch(&mut self) {
        self.status -= AccountStatus::Touched;
    }

    /// If account status is marked as touched.
    #[inline]
    pub fn is_touched(&self) -> bool {
        self.status.contains(AccountStatus::Touched)
    }

    /// Marks the account as newly created.
    #[inline]
    pub fn mark_created(&mut self) {
        self.status |= AccountStatus::Created;
    }

    /// Unmarks the created flag.
    #[inline]
    pub fn unmark_created(&mut self) {
        self.status -= AccountStatus::Created;
    }

    /// Marks the account as cold.
    #[inline]
    pub fn mark_cold(&mut self) {
        self.status |= AccountStatus::Cold;
    }

    /// Is account warm for given transaction id.
    #[inline]
    pub fn is_cold_transaction_id(&self, transaction_id: usize) -> bool {
        self.transaction_id != transaction_id || self.status.contains(AccountStatus::Cold)
    }

    /// Marks the account as warm and return true if it was previously cold.
    #[inline]
    pub fn mark_warm_with_transaction_id(&mut self, transaction_id: usize) -> bool {
        let is_cold = self.is_cold_transaction_id(transaction_id);
        self.status -= AccountStatus::Cold;
        self.transaction_id = transaction_id;
        is_cold
    }

    /// Is account locally created
    #[inline]
    pub fn is_created_locally(&self) -> bool {
        self.status.contains(AccountStatus::CreatedLocal)
    }

    /// Is account locally selfdestructed
    #[inline]
    pub fn is_selfdestructed_locally(&self) -> bool {
        self.status.contains(AccountStatus::SelfDestructedLocal)
    }

    /// Selfdestruct the account by clearing its storage and resetting its account info
    #[inline]
    pub fn selfdestruct(&mut self) {
        self.storage.clear();
        self.info = AccountInfo::default();
    }

    /// Mark account as locally created and mark global created flag.
    ///
    /// Returns true if it is created globally for first time.
    #[inline]
    pub fn mark_created_locally(&mut self) -> bool {
        self.status |= AccountStatus::CreatedLocal;
        let is_created_globaly = !self.status.contains(AccountStatus::Created);
        self.status |= AccountStatus::Created;
        is_created_globaly
    }

    /// Unmark account as locally created
    #[inline]
    pub fn unmark_created_locally(&mut self) {
        self.status -= AccountStatus::CreatedLocal;
    }

    /// Mark account as locally and globally selfdestructed
    #[inline]
    pub fn mark_selfdestructed_locally(&mut self) -> bool {
        self.status |= AccountStatus::SelfDestructedLocal;
        let is_global_selfdestructed = !self.status.contains(AccountStatus::SelfDestructed);
        self.status |= AccountStatus::SelfDestructed;
        is_global_selfdestructed
    }

    /// Unmark account as locally selfdestructed
    #[inline]
    pub fn unmark_selfdestructed_locally(&mut self) {
        self.status -= AccountStatus::SelfDestructedLocal;
    }

    /// Is account loaded as not existing from database.
    ///
    /// This is needed for pre spurious dragon hardforks where
    /// existing and empty were two separate states.
    pub fn is_loaded_as_not_existing(&self) -> bool {
        self.status.contains(AccountStatus::LoadedAsNotExisting)
    }

    /// Is account loaded as not existing from database and not touched.
    pub fn is_loaded_as_not_existing_not_touched(&self) -> bool {
        self.is_loaded_as_not_existing() && !self.is_touched()
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
    pub fn with_warm_mark(mut self, transaction_id: usize) -> (Self, bool) {
        let was_cold = self.mark_warm_with_transaction_id(transaction_id);
        (self, was_cold)
    }

    /// Variant of with_warm_mark that doesn't return the previous state.
    pub fn with_warm(mut self, transaction_id: usize) -> Self {
        self.mark_warm_with_transaction_id(transaction_id);
        self
    }
}

impl From<AccountInfo> for Account {
    fn from(info: AccountInfo) -> Self {
        Self {
            info,
            storage: HashMap::default(),
            transaction_id: 0,
            status: AccountStatus::empty(),
            original_info: AccountInfo::default(),
            bal: AccountBal::default(),
        }
    }
}

// The `bitflags!` macro generates `struct`s that manage a set of flags.
bitflags! {
    /// Account status flags. Generated by bitflags crate.
    ///
    /// With multi transaction feature there is a need to have both global and local fields.
    /// Global across multiple transaction and local across one transaction execution.
    ///
    /// Empty state without any flags set represent account that is loaded from db but not interacted with.
    ///
    /// `Touched` flag is used by database to check if account is potentially changed in some way.
    /// Additionally, after EIP-161 touch on empty-existing account would remove this account from state
    /// after transaction execution ends. Touch can span across multiple transactions as it is needed
    /// to be marked only once so it is safe to have only one global flag.
    /// Only first touch have different behaviour from others, and touch in first transaction will invalidate
    /// touch functionality in next transactions.
    ///
    /// `Created` flag is used to mark account as newly created in this transaction. This is used for optimization
    /// where if this flag is set we will not access database to fetch storage values.
    ///
    /// `CreatedLocal` flag is used after cancun to enable selfdestruct cleanup if account is created in same transaction.
    ///
    /// `Selfdestructed` flag is used to mark account as selfdestructed. On multiple calls this flag is preserved
    /// and on revert will stay selfdestructed.
    ///
    /// `SelfdestructLocal` is needed to award refund on first selfdestruct call. This flag is cleared on account loading.
    /// Over multiple transaction account can be selfdestructed in one tx, created in second tx and selfdestructed again in
    /// third tx.
    /// Additionally if account is loaded in second tx, storage and account that was destroyed in first tx needs to be cleared.
    ///
    /// `LoadedAsNotExisting` is used to mark account as loaded from database but with `balance == 0 && nonce == 0 && code = 0x`.
    /// This flag is fine to span across multiple transactions as it interucts with `Touched` flag this is used in global scope.
    ///
    /// `CreatedLocal`, `SelfdestructedLocal` and `Cold` flags are reset on first account loading of local scope.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
    #[cfg_attr(feature = "serde", serde(transparent))]
    pub struct AccountStatus: u8 {
        /// When account is newly created we will not access database
        /// to fetch storage values.
        const Created = 0b00000001;
        /// When accounts gets loaded this flag is set to false. Create will always be true if CreatedLocal is true.
        const CreatedLocal = 0b10000000;
        /// If account is marked for self destruction.
        const SelfDestructed = 0b00000010;
        /// If account is marked for self destruction.
        const SelfDestructedLocal = 0b01000000;
        /// Only when account is marked as touched we will save it to database.
        /// Additionally first touch on empty existing account (After EIP-161) will mark it
        /// for removal from state after transaction execution.
        const Touched = 0b00000100;
        /// used only for pre spurious dragon hardforks where existing and empty were two separate states.
        /// it became same state after EIP-161: State trie clearing
        const LoadedAsNotExisting = 0b00001000;
        /// used to mark account as cold.
        /// It is used only in local scope and it is reset on account loading.
        const Cold = 0b00010000;
    }
}

impl Default for AccountStatus {
    fn default() -> Self {
        AccountStatus::empty()
    }
}

/// This type keeps track of the current value of a storage slot.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EvmStorageSlot {
    /// Original value of the storage slot
    pub original_value: StorageValue,
    /// Present value of the storage slot
    pub present_value: StorageValue,
    /// Transaction id, used to track when storage slot was made warm.
    pub transaction_id: usize,
    /// Represents if the storage slot is cold
    pub is_cold: bool,
    /// BAL for storage slot.
    pub bal: BalWrites<StorageValue>,
}

impl EvmStorageSlot {
    /// Creates a new _unchanged_ `EvmStorageSlot` for the given value.
    pub fn new(original: StorageValue, transaction_id: usize) -> Self {
        Self {
            original_value: original,
            present_value: original,
            transaction_id,
            is_cold: false,
            bal: BalWrites::default(),
        }
    }

    /// Creates a new _changed_ `EvmStorageSlot`.
    pub fn new_changed(
        original_value: StorageValue,
        present_value: StorageValue,
        transaction_id: usize,
    ) -> Self {
        Self {
            original_value,
            present_value,
            transaction_id,
            is_cold: false,
            bal: BalWrites::default(),
        }
    }
    /// Returns true if the present value differs from the original value.
    pub fn is_changed(&self) -> bool {
        self.original_value != self.present_value
    }

    /// Returns the original value of the storage slot.
    #[inline]
    pub fn original_value(&self) -> StorageValue {
        self.original_value
    }

    /// Returns the current value of the storage slot.
    #[inline]
    pub fn present_value(&self) -> StorageValue {
        self.present_value
    }

    /// Marks the storage slot as cold. Does not change transaction_id.
    #[inline]
    pub fn mark_cold(&mut self) {
        self.is_cold = true;
    }

    /// Is storage slot cold for given transaction id.
    #[inline]
    pub fn is_cold_transaction_id(&self, transaction_id: usize) -> bool {
        self.transaction_id != transaction_id || self.is_cold
    }

    /// Marks the storage slot as warm and sets transaction_id to the given value
    ///
    ///
    /// Returns false if old transition_id is different from given id or in case they are same return `Self::is_cold` value.
    #[inline]
    pub fn mark_warm_with_transaction_id(&mut self, transaction_id: usize) -> bool {
        let is_cold = self.is_cold_transaction_id(transaction_id);
        self.transaction_id = transaction_id;
        self.is_cold = false;
        is_cold
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
        assert!(!account.mark_warm_with_transaction_id(0));

        // Mark account as cold
        account.mark_cold();

        // Account is cold
        assert!(account.status.contains(crate::AccountStatus::Cold));

        // When marking cold account as warm, it should return true
        assert!(account.mark_warm_with_transaction_id(0));
    }

    #[test]
    fn test_account_with_info() {
        let info = AccountInfo::default();
        let account = Account::default().with_info(info.clone());

        assert_eq!(account.info, info);
        assert_eq!(account.storage, HashMap::default());
        assert_eq!(account.status, AccountStatus::empty());
    }

    #[test]
    fn test_account_with_storage() {
        let mut storage = HashMap::<StorageKey, EvmStorageSlot>::default();
        let key1 = StorageKey::from(1);
        let key2 = StorageKey::from(2);
        let slot1 = EvmStorageSlot::new(StorageValue::from(10), 0);
        let slot2 = EvmStorageSlot::new(StorageValue::from(20), 0);

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
    fn test_storage_mark_warm_with_transaction_id() {
        let mut slot = EvmStorageSlot::new(U256::ZERO, 0);
        slot.is_cold = true;
        slot.transaction_id = 0;
        assert!(slot.mark_warm_with_transaction_id(1));

        slot.is_cold = false;
        slot.transaction_id = 0;
        assert!(slot.mark_warm_with_transaction_id(1));

        slot.is_cold = true;
        slot.transaction_id = 1;
        assert!(slot.mark_warm_with_transaction_id(1));

        slot.is_cold = false;
        slot.transaction_id = 1;
        // Only if transaction id is same and is_cold is false, return false.
        assert!(!slot.mark_warm_with_transaction_id(1));
    }

    #[test]
    fn test_account_with_warm_mark() {
        // Start with a cold account
        let cold_account = Account::default().with_cold_mark();
        assert!(cold_account.status.contains(AccountStatus::Cold));

        // Use with_warm_mark to warm it
        let (warm_account, was_cold) = cold_account.with_warm_mark(0);

        // Check that it's now warm and previously was cold
        assert!(!warm_account.status.contains(AccountStatus::Cold));
        assert!(was_cold);

        // Try with an already warm account
        let (still_warm_account, was_cold) = warm_account.with_warm_mark(0);
        assert!(!still_warm_account.status.contains(AccountStatus::Cold));
        assert!(!was_cold);
    }

    #[test]
    fn test_account_with_warm() {
        // Start with a cold account
        let cold_account = Account::default().with_cold_mark();
        assert!(cold_account.status.contains(AccountStatus::Cold));

        // Use with_warm to warm it
        let warm_account = cold_account.with_warm(0);

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
        let slot_value = EvmStorageSlot::new(StorageValue::from(123), 0);
        let mut storage = HashMap::<StorageKey, EvmStorageSlot>::default();
        storage.insert(slot_key, slot_value.clone());

        // Chain multiple builder methods together
        let account = Account::default()
            .with_info(info.clone())
            .with_storage(storage.into_iter())
            .with_created_mark()
            .with_touched_mark()
            .with_cold_mark()
            .with_warm(0);

        // Verify all modifications were applied
        assert_eq!(account.info, info);
        assert_eq!(account.storage.get(&slot_key), Some(&slot_value));
        assert!(account.is_created());
        assert!(account.is_touched());
        assert!(!account.status.contains(AccountStatus::Cold));
    }

    #[test]
    fn test_account_is_cold_transaction_id() {
        let mut account = Account::default();
        // only case where it is warm.
        assert!(!account.is_cold_transaction_id(0));

        // all other cases are cold
        assert!(account.is_cold_transaction_id(1));
        account.mark_cold();
        assert!(account.is_cold_transaction_id(0));
        assert!(account.is_cold_transaction_id(1));
    }
}
