use crate::{db::Database, primitives::Env};

/// EVM Data contains all the data that EVM needs to execute.
#[derive(Debug)]
pub struct EVMData<'a, DB: Database> {
    /// EVM Environment contains all the information about config, block and transaction that
    /// evm needs.
    pub env: &'a mut Env,
    /// Database to load data from.
    pub db: &'a mut DB,
}
