use revm_interpreter::primitives::{AccountInfo, Address, Bytecode, B256, U256};

/// Sorted accounts/storages/contracts for inclusion into database.
/// Structure is made so it is easier to apply directly to database
/// that mostly have separate tables to store account/storage/contract data.
#[derive(Clone, Debug, Default)]
pub struct StateChangeset {
    /// Vector of account presorted by address, with removed contracts bytecode
    pub accounts: Vec<(Address, Option<AccountInfo>)>,
    /// Vector of storage presorted by address
    /// First bool is indicator if storage needs to be dropped.
    pub storage: StorageChangeset,
    /// Vector of contracts presorted by bytecode hash
    pub contracts: Vec<(B256, Bytecode)>,
}

/// Storage changeset
pub type StorageChangeset = Vec<(Address, (bool, Vec<(U256, U256)>))>;

#[derive(Clone, Debug, Default)]
pub struct StateReverts {
    /// Vector of account presorted by address, with removed contracts bytecode
    ///
    /// Note: AccountInfo None means that account needs to be removed.
    pub accounts: Vec<Vec<(Address, Option<AccountInfo>)>>,
    /// Vector of storage presorted by address
    /// U256::ZERO means that storage needs to be removed.
    pub storage: StorageRevert,
}

impl StateReverts {
    /// Constructs new [StateReverts] with pre-allocated capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            accounts: Vec::with_capacity(capacity),
            storage: Vec::with_capacity(capacity),
        }
    }
}

/// Storage reverts
pub type StorageRevert = Vec<Vec<(Address, bool, Vec<(U256, U256)>)>>;
