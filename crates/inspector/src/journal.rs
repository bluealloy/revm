use auto_impl::auto_impl;
use revm::{
    context::JournaledState, database_interface::Database, primitives::Log, state::EvmState,
    JournalEntry,
};

pub trait JournalExt {
    fn logs(&self) -> &[Log];

    fn last_journal(&self) -> &[JournalEntry];

    fn evm_state(&self) -> &EvmState;

    fn evm_state_mut(&mut self) -> &mut EvmState;
}

impl<DB: Database> JournalExt for JournaledState<DB> {
    fn logs(&self) -> &[Log] {
        &self.logs
    }

    fn last_journal(&self) -> &[JournalEntry] {
        self.journal.last().expect("Journal is never empty")
    }

    fn evm_state(&self) -> &EvmState {
        &self.state
    }

    fn evm_state_mut(&mut self) -> &mut EvmState {
        &mut self.state
    }
}

#[auto_impl(&, &mut, Box, Arc)]
pub trait JournalExtGetter {
    type JournalExt: JournalExt;

    fn journal_ext(&self) -> &Self::JournalExt;
}
