use crate::{
    db::Database,
    primitives::{EVMError, Env},
};

/// EVM Data contains all the data that EVM needs to execute.
#[derive(Debug)]
pub struct EVMData<'a, DB: Database> {
    /// EVM Environment contains all the information about config, block and transaction that
    /// evm needs.
    pub env: &'a mut Env,
    /// Database to load data from.
    pub db: &'a mut DB,
}

impl<'a, DB: Database> EVMData<'a, DB> {
    /// Load access list for berlin hardfork.
    ///
    /// Loading of accounts/storages is needed to make them warm.
    #[inline]
    pub fn load_access_list(&mut self) -> Result<(), EVMError<DB::Error>> {
        todo!("not implemented yet")
    }

    /// Return environment.
    pub fn env(&mut self) -> &mut Env {
        self.env
    }
}
