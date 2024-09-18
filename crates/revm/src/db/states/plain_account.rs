use primitives::{HashMap, U256};
use state::{AccountInfo, EvmStorageSlot};

// TODO rename this to BundleAccount. As for the block level we have original state.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlainAccount {
    pub info: AccountInfo,
    pub storage: PlainStorage,
}

impl PlainAccount {
    pub fn new_empty_with_storage(storage: PlainStorage) -> Self {
        Self {
            info: AccountInfo::default(),
            storage,
        }
    }

    pub fn into_components(self) -> (AccountInfo, PlainStorage) {
        (self.info, self.storage)
    }
}

/// This type keeps track of the current value of a storage slot.
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, Hash)]
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

impl From<EvmStorageSlot> for StorageSlot {
    fn from(value: EvmStorageSlot) -> Self {
        Self::new_changed(value.original_value, value.present_value)
    }
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

/// This storage represent values that are before block changed.
///
/// Note: Storage that we get EVM contains original values before block changed.
pub type StorageWithOriginalValues = HashMap<U256, StorageSlot>;

/// Simple plain storage that does not have previous value.
/// This is used for loading from database, cache and for bundle state.
pub type PlainStorage = HashMap<U256, U256>;

impl From<AccountInfo> for PlainAccount {
    fn from(info: AccountInfo) -> Self {
        Self {
            info,
            storage: HashMap::new(),
        }
    }
}
