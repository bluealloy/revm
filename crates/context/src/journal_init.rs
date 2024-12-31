use super::journaled_state::JournaledState;
use database_interface::EmptyDB;

/// A clonable version of JournaledState that uses EmptyDB.
pub type JournalInit = JournaledState<EmptyDB>;

impl<DB> From<&JournaledState<DB>> for JournalInit {
    fn from(state: &JournaledState<DB>) -> Self {
        Self {
            database: EmptyDB::default(),
            state: state.state.clone(),
            transient_storage: state.transient_storage.clone(),
            logs: state.logs.clone(),
            depth: state.depth,
            journal: state.journal.clone(),
            spec: state.spec,
            warm_preloaded_addresses: state.warm_preloaded_addresses.clone(),
            precompiles: state.precompiles.clone(),
        }
    }
}
