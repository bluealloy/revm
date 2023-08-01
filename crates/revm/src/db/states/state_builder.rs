use super::{cache::CacheState, BundleState, State, TransitionState};
use crate::db::EmptyDB;
use core::convert::Infallible;
use revm_interpreter::primitives::db::Database;

/// Allows building of State and initializing it with different options.
pub struct StateBuilder<'a, DBError> {
    pub with_state_clear: bool,
    /// Optional database that we use to fetch data from. If database is not present, we will
    /// return not existing account and storage.
    ///
    /// Note: It is marked as Send so database can be shared between threads.
    pub database: Box<dyn Database<Error = DBError> + Send + 'a>,
    /// if there is prestate that we want to use.
    /// This would mean that we have additional state layer between evm and disk/database.
    pub with_bundle_prestate: Option<BundleState>,
    /// This will initialize cache to this state.
    pub with_cache_prestate: Option<CacheState>,
    /// Do we want to create reverts and update bundle state.
    /// Default is true.
    pub without_bundle_update: bool,
    /// Do we want to merge transitions in background.
    /// This will allows evm to continue executing.
    /// Default is false.
    pub with_background_transition_merge: bool,
}

impl Default for StateBuilder<'_, Infallible> {
    fn default() -> Self {
        Self {
            with_state_clear: true,
            database: Box::<EmptyDB>::default(),
            with_cache_prestate: None,
            with_bundle_prestate: None,
            without_bundle_update: false,
            with_background_transition_merge: false,
        }
    }
}

impl<'a, DBError> StateBuilder<'a, DBError> {
    /// Create default instance of builder.
    pub fn new() -> StateBuilder<'a, Infallible> {
        StateBuilder::<'a, Infallible>::default()
    }

    pub fn with_database<NewDBError>(
        self,
        database: Box<dyn Database<Error = NewDBError> + Send + 'a>,
    ) -> StateBuilder<'a, NewDBError> {
        // cast to the different database,
        // Note that we return different type depending of the database NewDBError.
        StateBuilder {
            with_state_clear: self.with_state_clear,
            database,
            with_cache_prestate: self.with_cache_prestate,
            with_bundle_prestate: self.with_bundle_prestate,
            without_bundle_update: self.without_bundle_update,
            with_background_transition_merge: self.with_background_transition_merge,
        }
    }

    /// By default state clear flag is enabled but for initial sync on mainnet
    /// we want to disable it so proper consensus changes are in place.
    pub fn without_state_clear(self) -> Self {
        Self {
            with_state_clear: false,
            ..self
        }
    }

    /// Allows setting prestate that is going to be used for execution.
    /// This bundle state will act as additional layer of cache.
    /// and State after not finding data inside StateCache will try to find it inside BundleState.
    ///
    /// On update Bundle state will be changed and updated.
    pub fn with_bundle_prestate(self, bundle: BundleState) -> Self {
        Self {
            with_bundle_prestate: Some(bundle),
            ..self
        }
    }

    /// Dont make transitions and dont update bundle state.
    ///
    /// This is good option if we dont care about creating reverts
    /// or getting output of changed states.
    pub fn without_bundle_update(self) -> Self {
        Self {
            without_bundle_update: true,
            ..self
        }
    }

    /// It will use different cache for the state. If set, it will ignore bundle prestate.
    /// and will ignore `without_state_clear` flag as cache contains its own state_clear flag.
    ///
    /// This is useful for testing.
    pub fn with_cached_prestate(self, cache: CacheState) -> Self {
        Self {
            with_cache_prestate: Some(cache),
            ..self
        }
    }

    /// Starts the thread that will take transitions and do merge to the bundle state
    /// in the background.
    pub fn with_background_transition_merge(self) -> Self {
        Self {
            with_background_transition_merge: true,
            ..self
        }
    }

    pub fn build(mut self) -> State<'a, DBError> {
        let use_preloaded_bundle = if self.with_cache_prestate.is_some() {
            self.with_bundle_prestate = None;
            false
        } else {
            self.with_bundle_prestate.is_some()
        };
        State {
            cache: self
                .with_cache_prestate
                .unwrap_or(CacheState::new(self.with_state_clear)),
            database: self.database,
            transition_state: if self.without_bundle_update {
                None
            } else {
                Some(TransitionState::default())
            },
            bundle_state: self.with_bundle_prestate,
            use_preloaded_bundle,
        }
    }
}
