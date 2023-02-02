//! Database that is split on State and BlockHash traits.
pub mod block_hash;
pub mod state;

pub use block_hash::{BlockHash, BlockHashRef};
pub use state::{State, StateRef};

use crate::{
    db::{Database, DatabaseRef},
    AccountInfo, Bytecode, B160, B256, U256,
};

pub struct DatabaseComponents<S, BH> {
    pub state: S,
    pub block_hash: BH,
}

pub enum DatabaseComponentError<SE, BHE> {
    State(SE),
    BlockHash(BHE),
}

impl<S: State, BH: BlockHash> Database for DatabaseComponents<S, BH> {
    type Error = DatabaseComponentError<S::Error, BH::Error>;

    fn basic(&mut self, address: B160) -> Result<Option<AccountInfo>, Self::Error> {
        self.state.basic(address).map_err(Self::Error::State)
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.state
            .code_by_hash(code_hash)
            .map_err(Self::Error::State)
    }

    fn storage(&mut self, address: B160, index: U256) -> Result<U256, Self::Error> {
        self.state
            .storage(address, index)
            .map_err(Self::Error::State)
    }

    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error> {
        self.block_hash
            .block_hash(number)
            .map_err(Self::Error::BlockHash)
    }
}

impl<S: StateRef, BH: BlockHashRef> DatabaseRef for DatabaseComponents<S, BH> {
    type Error = DatabaseComponentError<S::Error, BH::Error>;

    fn basic(&self, address: B160) -> Result<Option<AccountInfo>, Self::Error> {
        self.state.basic(address).map_err(Self::Error::State)
    }

    fn code_by_hash(&self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.state
            .code_by_hash(code_hash)
            .map_err(Self::Error::State)
    }

    fn storage(&self, address: B160, index: U256) -> Result<U256, Self::Error> {
        self.state
            .storage(address, index)
            .map_err(Self::Error::State)
    }

    fn block_hash(&self, number: U256) -> Result<B256, Self::Error> {
        self.block_hash
            .block_hash(number)
            .map_err(Self::Error::BlockHash)
    }
}
