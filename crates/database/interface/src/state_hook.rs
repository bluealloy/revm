//! State commit hook.

use crate::state::EvmState;

/// A hook that is called when state changes are committed.
pub trait OnStateHook: Send + 'static {
    /// Invoked with the state being committed.
    fn on_state(&mut self, state: EvmState);
}

impl<F> OnStateHook for F
where
    F: FnMut(EvmState) + Send + 'static,
{
    fn on_state(&mut self, state: EvmState) {
        self(state)
    }
}

/// An [`OnStateHook`] that does nothing.
#[derive(Default, Debug, Clone)]
#[non_exhaustive]
pub struct NoopHook;

impl OnStateHook for NoopHook {
    fn on_state(&mut self, _state: EvmState) {}
}
