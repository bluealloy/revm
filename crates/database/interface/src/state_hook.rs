//! State commit hook.

use crate::state::{AccountExtension, EvmState};

/// A hook that is called when state changes are committed.
pub trait OnStateHook<EXT: AccountExtension = ()>: Send + 'static {
    /// Invoked with the state being committed.
    fn on_state(&mut self, state: EvmState<EXT>);
}

impl<EXT: AccountExtension, F> OnStateHook<EXT> for F
where
    F: FnMut(EvmState<EXT>) + Send + 'static,
{
    fn on_state(&mut self, state: EvmState<EXT>) {
        self(state)
    }
}

/// An [`OnStateHook`] that does nothing.
#[derive(Default, Debug, Clone)]
#[non_exhaustive]
pub struct NoopHook;

impl<EXT: AccountExtension> OnStateHook<EXT> for NoopHook {
    fn on_state(&mut self, _state: EvmState<EXT>) {}
}
