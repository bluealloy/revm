use super::TransitionAccount;
use revm_interpreter::primitives::{hash_map::Entry, HashMap, StorageSlot, B160, U256};

/// TODO Rename this to become StorageWithOriginalValues or something like that.
/// This is used inside EVM and for block state. It is needed for block state to
/// be able to create changeset agains bundle state.
///
/// This storage represent values that are before block changed.
///
/// Note: Storage that we get EVM contains original values before t
pub type Storage = HashMap<U256, StorageSlot>;

#[derive(Clone, Debug)]
pub struct TransitionState {
    /// Block state account with account state
    pub transitions: HashMap<B160, TransitionAccount>,
    /// Has EIP-161 state clear enabled (Spurious Dragon hardfork).
    pub has_state_clear: bool,
}

impl Default for TransitionState {
    fn default() -> Self {
        // be default make state clear EIP enabled
        TransitionState {
            transitions: HashMap::new(),
            has_state_clear: true,
        }
    }
}

impl TransitionState {
    /// For newest fork this should be always `true`.
    ///
    /// For blocks before SpuriousDragon set this to `false`.
    pub fn new(has_state_clear: bool) -> Self {
        Self {
            transitions: HashMap::new(),
            has_state_clear,
        }
    }

    /// Return transition id and all account transitions. Leave empty transition map.
    pub fn take(&mut self) -> TransitionState {
        core::mem::take(self)
    }

    /// Used for tests only. When transitioned it is not recoverable
    pub fn set_state_clear(&mut self) {
        if self.has_state_clear == true {
            return;
        }

        self.has_state_clear = true;
    }

    pub fn add_transitions(&mut self, transitions: Vec<(B160, TransitionAccount)>) {
        for (address, account) in transitions {
            match self.transitions.entry(address) {
                Entry::Occupied(entry) => {
                    let entry = entry.into_mut();
                    entry.update(account);
                }
                Entry::Vacant(entry) => {
                    entry.insert(account);
                }
            }
        }
    }
}
