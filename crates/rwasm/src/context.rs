use crate::primitives::Env;

/// EVM Data contains all the data that EVM needs to execute.
#[derive(Debug)]
pub struct EVMData<'a> {
    /// EVM Environment contains all the information about config, block and transaction that
    /// evm needs.
    pub env: &'a mut Env,
}
