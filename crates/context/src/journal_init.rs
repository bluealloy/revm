use super::journaled_state::JournaledState;
use database_interface::EmptyDB;

/// A clonable version of JournaledState that uses EmptyDB.
pub type JournalInit = JournaledState<EmptyDB>;

impl<DB> JournaledState<DB> {
    pub fn into_init(self) -> JournalInit {
        JournalInit {
            database: EmptyDB::default(),
            state: self.state,
            transient_storage: self.transient_storage,
            logs: self.logs,
            depth: self.depth,
            journal: self.journal,
            spec: self.spec,
            warm_preloaded_addresses: self.warm_preloaded_addresses,
            precompiles: self.precompiles,
        }
    }

    pub fn to_init(&self) -> JournalInit {
        JournalInit {
            database: EmptyDB::default(),
            state: self.state.clone(),
            transient_storage: self.transient_storage.clone(),
            logs: self.logs.clone(),
            depth: self.depth,
            journal: self.journal.clone(),
            spec: self.spec,
            warm_preloaded_addresses: self.warm_preloaded_addresses.clone(),
            precompiles: self.precompiles.clone(),
        }
    }
}
