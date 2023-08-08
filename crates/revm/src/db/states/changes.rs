use revm_interpreter::primitives::{AccountInfo, Bytecode, B160, B256, U256};

/// Sorted accounts/storages/contracts for inclusion into database.
/// Structure is made so it is easier to apply dirrectly to database
/// that mostly have saparate tables to store account/storage/contract data.
#[derive(Clone, Debug, Default)]
pub struct StateChangeset {
    /// Vector of account presorted by address, with removed contracts bytecode
    pub accounts: Vec<(B160, Option<AccountInfo>)>,
    /// Vector of storage presorted by address
    /// First bool is indicatior if storage needs to be dropped.
    pub storage: StorageChangeset,
    /// Vector of contracts presorted by bytecode hash
    pub contracts: Vec<(B256, Bytecode)>,
}

/// Storage changeset
pub type StorageChangeset = Vec<(B160, (bool, Vec<(U256, U256)>))>;

#[derive(Clone, Debug, Default)]
pub struct StateReverts {
    /// Vector of account presorted by anddress, with removed cotracts bytecode
    ///
    /// Note: AccountInfo None means that account needs to be removed.
    pub accounts: Vec<Vec<(B160, Option<AccountInfo>)>>,
    /// Vector of storage presorted by address
    /// U256::ZERO means that storage needs to be removed.
    pub storage: StorageRevert,
}

/// Storage reverts
pub type StorageRevert = Vec<Vec<(B160, bool, Vec<(U256, U256)>)>>;
