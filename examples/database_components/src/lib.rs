//! Database component example.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

//! Database that is split on State and BlockHash traits.
pub mod block_hash;
pub mod state;

pub use block_hash::{BlockHash, BlockHashRef};
pub use state::{State, StateRef};

use revm::{
    database_interface::{DBErrorMarker, Database, DatabaseCommit, DatabaseRef},
    primitives::{Address, AddressMap, StorageKey, StorageValue, B256},
    state::{Account, AccountInfo, Bytecode},
};

/// A database implementation that separates state and block hash components.
/// This allows for modular database design where state and block hash
/// functionality can be implemented independently.
#[derive(Debug)]
pub struct DatabaseComponents<S, BH> {
    /// State component for account and storage operations
    pub state: S,
    /// Block hash component for retrieving historical block hashes
    pub block_hash: BH,
}

/// Error type for database component operations.
/// Wraps errors from both state and block hash components.
#[derive(Debug, thiserror::Error)]
pub enum DatabaseComponentError<
    SE: core::error::Error + Send + Sync + 'static,
    BHE: core::error::Error + Send + Sync + 'static,
> {
    /// Error from state component operations
    #[error(transparent)]
    State(SE),
    /// Error from block hash component operations
    #[error(transparent)]
    BlockHash(BHE),
}

impl<
        SE: core::error::Error + Send + Sync + 'static,
        BHE: core::error::Error + Send + Sync + 'static,
    > DBErrorMarker for DatabaseComponentError<SE, BHE>
{
}

unsafe impl<
        SE: core::error::Error + Send + Sync + 'static,
        BHE: core::error::Error + Send + Sync + 'static,
    > Send for DatabaseComponentError<SE, BHE>
{
}

impl<S: State, BH: BlockHash> Database for DatabaseComponents<S, BH> {
    type Error = DatabaseComponentError<S::Error, BH::Error>;

    fn basic(&mut self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        self.state.basic(address).map_err(Self::Error::State)
    }

    fn code_by_hash(&mut self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.state
            .code_by_hash(code_hash)
            .map_err(Self::Error::State)
    }

    fn storage(
        &mut self,
        address: Address,
        index: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        self.state
            .storage(address, index)
            .map_err(Self::Error::State)
    }

    fn block_hash(&mut self, number: u64) -> Result<B256, Self::Error> {
        self.block_hash
            .block_hash(number)
            .map_err(Self::Error::BlockHash)
    }
}

impl<S: StateRef, BH: BlockHashRef> DatabaseRef for DatabaseComponents<S, BH> {
    type Error = DatabaseComponentError<S::Error, BH::Error>;

    fn basic_ref(&self, address: Address) -> Result<Option<AccountInfo>, Self::Error> {
        self.state.basic(address).map_err(Self::Error::State)
    }

    fn code_by_hash_ref(&self, code_hash: B256) -> Result<Bytecode, Self::Error> {
        self.state
            .code_by_hash(code_hash)
            .map_err(Self::Error::State)
    }

    fn storage_ref(
        &self,
        address: Address,
        index: StorageKey,
    ) -> Result<StorageValue, Self::Error> {
        self.state
            .storage(address, index)
            .map_err(Self::Error::State)
    }

    fn block_hash_ref(&self, number: u64) -> Result<B256, Self::Error> {
        self.block_hash
            .block_hash(number)
            .map_err(Self::Error::BlockHash)
    }
}

impl<S: DatabaseCommit, BH: BlockHashRef> DatabaseCommit for DatabaseComponents<S, BH> {
    fn commit(&mut self, changes: AddressMap<Account>) {
        self.state.commit(changes);
    }
}
