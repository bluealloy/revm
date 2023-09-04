use super::RevertToSlot;
use revm_interpreter::primitives::{AccountInfo, Bytecode, B160, B256, U256};

/// Sorted accounts/storages/contracts for inclusion into database.
/// Structure is made so it is easier to apply directly to database
/// that mostly have separate tables to store account/storage/contract data.
#[derive(Clone, Debug, Default)]
pub struct StateChangeset {
    /// Vector of account presorted by address, with removed contracts bytecode
    pub accounts: Vec<(B160, Option<AccountInfo>)>,
    /// Vector of storage presorted by address
    /// First bool is indicator if storage needs to be dropped.
    pub storage: Vec<PlainStorageChangeset>,
    /// Vector of contracts presorted by bytecode hash
    pub contracts: Vec<(B256, Bytecode)>,
}

/// Plain storage changeset. Used to apply storage changes of plain state to
/// the database.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct PlainStorageChangeset {
    /// Address of account
    pub address: B160,
    /// Storage key value pairs.
    pub storage: Vec<(U256, U256)>,
}

/// Plain Storage Revert. Containing old values of changed storage.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct PlainStorageRevert {
    /// Address of account
    pub address: B160,
    /// Is storage wiped in this revert. Wiped flag is set on
    /// first known selfdestruct and would require clearing the
    /// state of this storage from database (And moving it to revert).
    pub wiped: bool,
    /// Contains the storage key and old values of that storage.
    /// Assume they are sorted by the key.
    pub storage_revert: Vec<(U256, RevertToSlot)>,
}

/// Plain state reverts are used to easily store reverts into database.
///
/// Note that accounts are assumed sorted by address.
#[derive(Clone, Debug, Default)]
pub struct PlainStateReverts {
    /// Vector of account presorted by address, with removed contracts bytecode
    ///
    /// Note: AccountInfo None means that account needs to be removed.
    pub accounts: Vec<Vec<(B160, Option<AccountInfo>)>>,
    /// Vector of storage presorted by address
    pub storage: Vec<Vec<PlainStorageRevert>>,
}

impl PlainStateReverts {
    /// Constructs new [StateReverts] with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            accounts: Vec::with_capacity(capacity),
            storage: Vec::with_capacity(capacity),
        }
    }
}

/// Storage reverts
pub type StorageRevert = Vec<Vec<(B160, bool, Vec<(U256, RevertToSlot)>)>>;
