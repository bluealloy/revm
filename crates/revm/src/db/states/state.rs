use core::convert::Infallible;

use super::{cache::CacheState, plain_account::PlainStorage, BundleState, TransitionState};
use crate::{db::EmptyDB, TransitionAccount};
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

impl TransitionBuilder {
    /// Take all transitions and merge them inside bundle state.
    /// This action will create final post state and all reverts so that
    /// we at any time revert state of bundle to the state before transition
    /// is applied.
    pub fn merge_transitions(&mut self) {
        let transition_state = self.transition_state.take();
        self.bundle_state
            .apply_block_substate_and_create_reverts(transition_state);
    }
}

/// For State that does not have database.
impl State<Infallible> {
    pub fn new_with_cache(mut cache: CacheState, has_state_clear: bool) -> Self {
        cache.has_state_clear = has_state_clear;
        Self {
            cache,
            database: Box::new(EmptyDB::default()),
            transition_builder: None,
            has_state_clear,
        }
    }

    pub fn new_cached_with_transition() -> Self {
        Self {
            cache: CacheState::default(),
            database: Box::new(EmptyDB::default()),
            transition_builder: Some(TransitionBuilder {
                transition_state: TransitionState::new(true),
                bundle_state: BundleState::default(),
            }),
            has_state_clear: true,
        }
    }

    pub fn new() -> Self {
        Self {
            cache: CacheState::default(),
            database: Box::new(EmptyDB::default()),
            transition_builder: None,
            has_state_clear: true,
        }
    }

    pub fn new_legacy() -> Self {
        Self {
            cache: CacheState::default(),
            database: Box::new(EmptyDB::default()),
            transition_builder: None,
            has_state_clear: false,
        }
    }
}

impl<DBError> State<DBError> {
    /// State clear EIP-161 is enabled in Spurious Dragon hardfork.
    pub fn enable_state_clear_eip(&mut self) {
        self.has_state_clear = true;
        self.cache.has_state_clear = true;
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

    pub fn insert_not_existing(&mut self, address: B160) {
        self.cache.insert_not_existing(address)
    }

    pub fn insert_account(&mut self, address: B160, info: AccountInfo) {
        self.cache.insert_account(address, info)
    }

    pub fn insert_account_with_storage(
        &mut self,
        address: B160,
        info: AccountInfo,
        storage: PlainStorage,
    ) {
        self.cache
            .insert_account_with_storage(address, info, storage)
    }

    /// Apply evm transitions to transition state.
    pub fn apply_transition(&mut self, transitions: Vec<(B160, TransitionAccount)>) {
        // add transition to transition state.
        if let Some(transition_builder) = self.transition_builder.as_mut() {
            // NOTE: can be done in parallel
            transition_builder
                .transition_state
                .add_transitions(transitions);
        }
    }

    pub fn merge_transitions(&mut self) {
        if let Some(builder) = self.transition_builder.as_mut() {
            builder.merge_transitions()
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

impl<DBError> DatabaseCommit for State<DBError> {
    fn commit(&mut self, evm_state: HashMap<B160, Account>) {
        let transitions = self.cache.apply_evm_state(evm_state);
        self.apply_transition(transitions);
    }
}
