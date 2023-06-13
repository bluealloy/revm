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
    pub storage: Vec<(B160, (bool, Vec<(U256, U256)>))>,
    /// Vector of contracts presorted by bytecode hash
    pub contracts: Vec<(B256, Bytecode)>,
}

pub struct StateReverts {
    /// Vector of account presorted by anddress, with removed cotracts bytecode
    ///
    /// Note: AccountInfo None means that account needs to be removed.
    pub accounts: Vec<(B160, Option<AccountInfo>)>,
    /// Vector of storage presorted by address
    /// U256::ZERO means that storage needs to be removed.
    pub storage: Vec<(B160, Vec<(U256, U256)>)>,
    /// Vector of contracts presorted by bytecode hash
    ///
    /// TODO: u64 counter is still not used. but represent number of times this contract was
    /// created, as multiple accounts can create same contract bytes.
    pub contracts: Vec<(B256, (u64, Bytecode))>,
}
