mod in_memory_db;

#[cfg(feature = "ethersdb")]
pub mod ethersdb;

#[cfg(all(not(feature = "ethersdb"), feature = "web3db"))]
compile_error!(
    "`web3db` feature is deprecated, drop-in replacement can be found with feature `ethersdb`"
);

#[cfg(feature = "ethersdb")]
pub use ethersdb::EthersDB;
use hashbrown::HashMap as Map;
use revm_interpreter::{Account, AccountInfo, Bytecode, B160, B256, U256};

use crate::{
    blockchain::{BlockHash, BlockHashRef},
    state::{State, StateRef},
    StateCommit,
};

pub use self::in_memory_db::{AccountState, BenchmarkDB, CacheDB, DbAccount, EmptyDB, InMemoryDB};

#[impl_tools::autoimpl(for<T: trait> &mut T, Box<T>)]
pub trait Database: BlockHash + State {
    type DatabaseError: From<<Self as BlockHash>::Error> + From<<Self as State>::Error>;
}

#[impl_tools::autoimpl(for<T: trait> &T, Box<T>)]
#[cfg_attr(feature = "std", impl_tools::autoimpl(for<T: trait> std::sync::Arc<T>))]
pub trait DatabaseRef: BlockHashRef + StateRef {
    type DatabaseError: From<<Self as BlockHashRef>::Error> + From<<Self as StateRef>::Error>;
}

impl<T> Database for &T
where
    T: DatabaseRef,
{
    type DatabaseError = <T as DatabaseRef>::DatabaseError;
}

pub struct DatabaseComponents<BH: BlockHash, S: State> {
    pub block_hash: BH,
    pub state: S,
}

pub enum ComponentError<BHE, SE> {
    BlockHashError(BHE),
    StateError(SE),
}

impl<BH: BlockHash, S: State> Database for DatabaseComponents<BH, S> {
    type DatabaseError = ComponentError<BH::Error, S::Error>;
}

impl<BH: BlockHash, S: State> BlockHash for DatabaseComponents<BH, S> {
    type Error = ComponentError<BH::Error, S::Error>;

    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error> {
        self.block_hash
            .block_hash(number)
            .map_err(ComponentError::BlockHashError)
    }
}

impl<BH: BlockHash, S: State> State for DatabaseComponents<BH, S> {
    type Error = ComponentError<BH::Error, S::Error>;

    fn basic(&mut self, address: B160) -> Result<Option<AccountInfo>, Self::Error> {
        self.state
            .basic(address)
            .map_err(ComponentError::StateError)
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.state
            .code_by_hash(code_hash)
            .map_err(ComponentError::StateError)
    }

    fn storage(&mut self, address: B160, index: U256) -> Result<U256, Self::Error> {
        self.state
            .storage(address, index)
            .map_err(ComponentError::StateError)
    }
}

impl<BH: BlockHash, S: State + StateCommit> StateCommit for DatabaseComponents<BH, S> {
    fn commit(&mut self, changes: Map<B160, Account>) {
        self.state.commit(changes)
    }
}
