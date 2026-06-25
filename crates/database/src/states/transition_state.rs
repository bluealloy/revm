use std::borrow::Cow;

use super::{StorageSlot, TransitionAccount};
use primitives::{hash_map::Entry, Address, AddressMap, HashMap};
use state::EvmStorage;

/// State of accounts in transition between transaction executions.
#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct TransitionState {
    /// Block state account with account state
    pub transitions: AddressMap<TransitionAccount>,
}

impl TransitionState {
    /// Create new transition state containing one [`TransitionAccount`].
    pub fn single(address: Address, transition: TransitionAccount) -> Self {
        let mut transitions = HashMap::default();
        transitions.insert(address, transition);
        TransitionState { transitions }
    }

    /// Take the contents of this [`TransitionState`] and replace it with an
    /// empty one.
    ///
    /// See [core::mem::take].
    pub fn take(&mut self) -> TransitionState {
        core::mem::take(self)
    }

    /// Clear the transition state.
    pub fn clear(&mut self) {
        self.transitions.clear();
    }

    /// Add transitions to the transition state.
    ///
    /// This will insert new [`TransitionAccount`]s, or update existing ones via
    /// [`update`][TransitionAccount::update].
    pub fn add_transitions<'a>(
        &mut self,
        transitions: impl IntoIterator<Item = (Address, TransitionAccount<Option<Cow<'a, EvmStorage>>>)>,
    ) {
        let transitions = transitions.into_iter();
        if let Some(upper) = transitions.size_hint().1 {
            self.transitions.reserve(upper);
        }
        for (address, account) in transitions {
            self.add_transition(address, account);
        }
    }

    /// Add one transition to the transition state.
    pub fn add_transition(
        &mut self,
        address: Address,
        account: TransitionAccount<Option<Cow<'_, EvmStorage>>>,
    ) {
        match self.transitions.entry(address) {
            Entry::Occupied(entry) => entry.into_mut().update(account),
            Entry::Vacant(entry) => {
                _ = entry.insert(account.map_storage(|storage| {
                    storage
                        .map(|storage| {
                            storage
                                .iter()
                                .filter_map(|(key, slot)| {
                                    slot.is_changed().then_some((
                                        *key,
                                        StorageSlot::new_changed(
                                            slot.original_value,
                                            slot.present_value,
                                        ),
                                    ))
                                })
                                .collect()
                        })
                        .unwrap_or_default()
                }))
            }
        }
    }
}
