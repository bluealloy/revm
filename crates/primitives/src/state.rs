use crate::{Address, Bytecode, HashMap, B256, KECCAK_EMPTY, U256};
use bitflags::bitflags;
use core::hash::{Hash, Hasher};

/// EVM State is a mapping from addresses to accounts.
pub type State = HashMap<Address, Account>;

/// Structure used for EIP-1153 transient storage.
pub type TransientStorage = HashMap<(Address, U256), U256>;

/// An account's Storage is a mapping from 256-bit integer keys to [StorageSlot]s.
pub type Storage = HashMap<U256, StorageSlot>;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Account {
    /// Balance, nonce, and code.
    pub info: AccountInfo,
    /// Storage cache
    pub storage: Storage,
    /// Account status flags.
    pub status: AccountStatus,
}

// The `bitflags!` macro generates `struct`s that manage a set of flags.
bitflags! {
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
    }
}

impl Default for AccountStatus {
    fn default() -> Self {
        Self::Loaded
    }
}

impl Account {
    /// Create new account and mark it as non existing.
    pub fn new_not_existing() -> Self {
        Self {
            info: AccountInfo::default(),
            storage: HashMap::new(),
            status: AccountStatus::LoadedAsNotExisting,
        }
    }

    /// Mark account as self destructed.
    pub fn mark_selfdestruct(&mut self) {
        self.status |= AccountStatus::SelfDestructed;
    }

    /// Unmark account as self destructed.
    pub fn unmark_selfdestruct(&mut self) {
        self.status -= AccountStatus::SelfDestructed;
    }

    /// Is account marked for self destruct.
    pub fn is_selfdestructed(&self) -> bool {
        self.status.contains(AccountStatus::SelfDestructed)
    }

    /// Mark account as touched
    pub fn mark_touch(&mut self) {
        self.status |= AccountStatus::Touched;
    }

    /// Unmark the touch flag.
    pub fn unmark_touch(&mut self) {
        self.status -= AccountStatus::Touched;
    }

    /// If account status is marked as touched.
    pub fn is_touched(&self) -> bool {
        self.status.contains(AccountStatus::Touched)
    }

    /// Mark account as newly created.
    pub fn mark_created(&mut self) {
        self.status |= AccountStatus::Created;
    }

    /// Unmark created flag.
    pub fn unmark_created(&mut self) {
        self.status -= AccountStatus::Created;
    }

    /// Is account loaded as not existing from database
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
    /// See also [StorageSlot::is_changed]
    pub fn changed_storage_slots(&self) -> impl Iterator<Item = (&U256, &StorageSlot)> {
        self.storage.iter().filter(|(_, slot)| slot.is_changed())
    }
}

impl From<AccountInfo> for Account {
    fn from(info: AccountInfo) -> Self {
        Self {
            info,
            storage: HashMap::new(),
            status: AccountStatus::Loaded,
        }
    }
}

/// This type keeps track of the current value of a storage slot.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StorageSlot {
    /// The value of the storage slot before it was changed.
    ///
    /// When the slot is first loaded, this is the original value.
    ///
    /// If the slot was not changed, this is equal to the present value.
    pub previous_or_original_value: U256,
    /// When loaded with sload present value is set to original value
    pub present_value: U256,
}

impl StorageSlot {
    /// Creates a new _unchanged_ `StorageSlot` for the given value.
    pub fn new(original: U256) -> Self {
        Self {
            previous_or_original_value: original,
            present_value: original,
        }
    }

    /// Creates a new _changed_ `StorageSlot`.
    pub fn new_changed(previous_or_original_value: U256, present_value: U256) -> Self {
        Self {
            previous_or_original_value,
            present_value,
        }
    }

    /// Returns true if the present value differs from the original value
    pub fn is_changed(&self) -> bool {
        self.previous_or_original_value != self.present_value
    }

    /// Returns the original value of the storage slot.
    pub fn original_value(&self) -> U256 {
        self.previous_or_original_value
    }

    /// Returns the current value of the storage slot.
    pub fn present_value(&self) -> U256 {
        self.present_value
    }
}

/// AccountInfo account information.
#[derive(Clone, Debug, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AccountInfo {
    /// Account balance.
    pub balance: U256,
    /// Account nonce.
    pub nonce: u64,
    /// code hash,
    pub code_hash: B256,
    /// code: if None, `code_by_hash` will be used to fetch it if code needs to be loaded from
    /// inside of `revm`.
    pub code: Option<Bytecode>,
}

impl Default for AccountInfo {
    fn default() -> Self {
        Self {
            balance: U256::ZERO,
            code_hash: KECCAK_EMPTY,
            code: Some(Bytecode::new()),
            nonce: 0,
        }
    }
}

impl PartialEq for AccountInfo {
    fn eq(&self, other: &Self) -> bool {
        self.balance == other.balance
            && self.nonce == other.nonce
            && self.code_hash == other.code_hash
    }
}

impl Hash for AccountInfo {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.balance.hash(state);
        self.nonce.hash(state);
        self.code_hash.hash(state);
    }
}

impl AccountInfo {
    pub fn new(balance: U256, nonce: u64, code_hash: B256, code: Bytecode) -> Self {
        Self {
            balance,
            nonce,
            code: Some(code),
            code_hash,
        }
    }

    /// Returns account info without the code.
    pub fn without_code(mut self) -> Self {
        self.take_bytecode();
        self
    }

    /// Returns if an account is empty.
    ///
    /// An account is empty if the following conditions are met.
    /// - code hash is zero or set to the Keccak256 hash of the empty string `""`
    /// - balance is zero
    /// - nonce is zero
    pub fn is_empty(&self) -> bool {
        let code_empty = self.is_empty_code_hash() || self.code_hash == B256::ZERO;
        code_empty && self.balance == U256::ZERO && self.nonce == 0
    }

    /// Returns `true` if the account is not empty.
    pub fn exists(&self) -> bool {
        !self.is_empty()
    }

    /// Returns `true` if account has no nonce and code.
    pub fn has_no_code_and_nonce(&self) -> bool {
        self.is_empty_code_hash() && self.nonce == 0
    }

    /// Return bytecode hash associated with this account.
    /// If account does not have code, it return's `KECCAK_EMPTY` hash.
    pub fn code_hash(&self) -> B256 {
        self.code_hash
    }

    /// Returns true if the code hash is the Keccak256 hash of the empty string `""`.
    #[inline]
    pub fn is_empty_code_hash(&self) -> bool {
        self.code_hash == KECCAK_EMPTY
    }

    /// Take bytecode from account. Code will be set to None.
    pub fn take_bytecode(&mut self) -> Option<Bytecode> {
        self.code.take()
    }

    pub fn from_balance(balance: U256) -> Self {
        AccountInfo {
            balance,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{Account, KECCAK_EMPTY, U256};

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
}
