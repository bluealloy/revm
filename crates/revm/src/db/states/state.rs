use core::convert::Infallible;

use super::{cache::CacheState, BundleState, TransitionState};
use crate::db::EmptyDB;
use revm_interpreter::primitives::{
    db::{Database, DatabaseCommit},
    Account, AccountInfo, Bytecode, HashMap, B160, B256, U256,
};

/// State of blockchain.
pub struct State<DBError> {
    /// Cached state contains both changed from evm executiong and cached/loaded account/storages
    /// from database. This allows us to have only one layer of cache where we can fetch data.
    /// Additionaly we can introuduce some preloading of data from database.
    pub cache: CacheState,
    /// Optional database that we use to fetch data from. If database is not present, we will
    /// return not existing account and storage.
    pub database: Box<dyn Database<Error = DBError>>,
    /// Build reverts and state that gets applied to the state.
    pub transition_builder: Option<TransitionBuilder>,
    /// Is state clear enabled
    /// TODO: should we do it as block number, it would be easier.
    pub has_state_clear: bool,
}

#[derive(Debug, Clone, Default)]
pub struct TransitionBuilder {
    /// Block state, it aggregates transactions transitions into one state.
    pub transition_state: TransitionState,
    /// After block is finishes we merge those changes inside bundle.
    /// Bundle is used to update database and create changesets.
    pub bundle_state: BundleState,
}

/// For State that does not have database.
impl State<Infallible> {
    pub fn new_cached() -> Self {
        Self {
            cache: CacheState::default(),
            database: Box::new(EmptyDB::default()),
            transition_builder: None,
            has_state_clear: true,
        }
    }

    pub fn new_cached_with_transition() -> Self {
        let db = Box::new(EmptyDB::default());
        Self {
            cache: CacheState::default(),
            database: db,
            transition_builder: Some(TransitionBuilder {
                transition_state: TransitionState::new(false),
                bundle_state: BundleState::default(),
            }),
            has_state_clear: true,
        }
    }

    pub fn new() -> Self {
        let db = Box::new(EmptyDB::default());
        Self {
            cache: CacheState::default(),
            database: db,
            transition_builder: None,
            has_state_clear: true,
        }
    }
}

impl<DBError> State<DBError> {
    /// State clear EIP-161 is enabled in Spurious Dragon hardfork.
    pub fn enable_state_clear_eip(&mut self) {
        self.has_state_clear = true;
        self.transition_builder
            .as_mut()
            .map(|t| t.transition_state.set_state_clear());
    }

    pub fn new_with_transtion(db: Box<dyn Database<Error = DBError>>) -> Self {
        Self {
            cache: CacheState::default(),
            database: db,
            transition_builder: Some(TransitionBuilder {
                transition_state: TransitionState::new(false),
                bundle_state: BundleState::default(),
            }),
            has_state_clear: true,
        }
    }
}

impl<DBError> Database for State<DBError> {
    type Error = DBError;

    fn basic(&mut self, address: B160) -> Result<Option<AccountInfo>, Self::Error> {
        // get from cache
        if let Some(account) = self.cache.accounts.get(&address) {
            return Ok(account.account_info());
        }

        self.database.basic(address)
    }

    fn code_by_hash(
        &mut self,
        code_hash: revm_interpreter::primitives::B256,
    ) -> Result<Bytecode, Self::Error> {
        self.database.code_by_hash(code_hash)
    }

    fn storage(&mut self, address: B160, index: U256) -> Result<U256, Self::Error> {
        // get from cache
        if let Some(account) = self.cache.accounts.get(&address) {
            return Ok(account.storage_slot(index).unwrap_or_default());
        }

        self.database.storage(address, index)
    }

    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error> {
        self.database.block_hash(number)
    }
}

impl<DB: Database> DatabaseCommit for State<DB> {
    fn commit(&mut self, evm_state: HashMap<B160, Account>) {
        let transitions = self.cache.apply_evm_state(evm_state);
        // add transition to transition state.
        if let Some(transition_builder) = self.transition_builder.as_mut() {
            transition_builder
                .transition_state
                .add_transitions(transitions);
        }
    }
}
