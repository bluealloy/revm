use super::TransitionAccount;
use alloc::vec::Vec;
use revm_interpreter::primitives::{hash_map::Entry, Address, HashMap};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransitionState {
    /// Block state account with account state
    pub transitions: HashMap<Address, TransitionAccount>,
}

impl Default for TransitionState {
    fn default() -> Self {
        // be default make state clear EIP enabled
        TransitionState {
            transitions: HashMap::new(),
        }
    }
}

impl TransitionState {
    /// Create new transition state with one transition.
    pub fn single(address: Address, transition: TransitionAccount) -> Self {
        let mut transitions = HashMap::new();
        transitions.insert(address, transition);
        TransitionState { transitions }
    }

    /// Return transition id and all account transitions. Leave empty transition map.
    pub fn take(&mut self) -> TransitionState {
        core::mem::take(self)
    }

    pub fn add_transitions(&mut self, transitions: Vec<(Address, TransitionAccount)>) {
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
