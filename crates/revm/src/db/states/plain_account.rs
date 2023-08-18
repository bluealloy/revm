use revm_interpreter::primitives::{AccountInfo, HashMap, StorageSlot, U256};

/// TODO rename this to BundleAccount. As for the block level we have original state.
#[derive(Clone, Debug, Default)]
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

/// TODO Rename this to become StorageWithOriginalValues or something like that.
/// This is used inside EVM and for block state. It is needed for block state to
/// be able to create changeset agains bundle state.
///
/// This storage represent values that are before block changed.
///
/// Note: Storage that we get EVM contains original values before t
pub type StorageWithOriginalValues = HashMap<U256, StorageSlot>;

/// Simple plain storage that does not have previous value.
/// This is used for loading from database, cache and for bundle state.
///
pub type PlainStorage = HashMap<U256, U256>;

impl From<AccountInfo> for PlainAccount {
    fn from(info: AccountInfo) -> Self {
        Self {
            info,
            storage: HashMap::new(),
        }
    }
}
