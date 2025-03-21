use super::Journal;
use database_interface::EmptyDB;

/// A cloneable version of JournaledState that uses EmptyDB.
/// Used to clone the journaled state and for initialization of new journaled state.
pub type JournalInit = Journal<EmptyDB>;

impl<DB> Journal<DB> {
    /// Creates a new JournalInit by moving all internal state data (state, storage, logs, etc) into a new
    /// journal with an empty database. This consumes the original journal.
    ///
    /// This is useful when you want to transfer the current state to a new execution context that doesn't
    /// need access to the original database, like when snapshotting state or forking execution.
    ///
    /// If you need to preserve the original journal, use [`Self::to_init`] instead which clones the state.
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

    /// Creates a new JournalInit by cloning all internal state data (state, storage, logs, etc)
    /// but using an empty database. This allows creating a new journaled state with the same
    /// state data but without carrying over the original database.
    ///
    /// This is useful when you want to reuse the current state for a new transaction or
    /// execution context, but want to start with a fresh database.
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
