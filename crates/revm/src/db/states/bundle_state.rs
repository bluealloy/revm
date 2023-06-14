use super::{
    changes::StateChangeset, reverts::AccountInfoRevert, AccountRevert, BundleAccount,
    StateReverts, TransitionState,
};
use rayon::slice::ParallelSliceMut;
use revm_interpreter::primitives::{hash_map, Bytecode, HashMap, B160, B256};

// TODO
#[derive(Clone, Debug)]
pub struct BundleState {
    /// State.
    pub state: HashMap<B160, BundleAccount>,
    /// All created contracts in this block.
    pub contracts: HashMap<B256, Bytecode>,
    /// Changes to revert
    pub reverts: Vec<Vec<(B160, AccountRevert)>>,
}

impl Default for BundleState {
    fn default() -> Self {
        Self {
            state: HashMap::new(),
            reverts: Vec::new(),
            contracts: HashMap::new(),
        }
    }
}

impl BundleState {
    // Consume `TransitionState` by applying the changes and creating the reverts
    pub fn apply_block_substate_and_create_reverts(&mut self, mut transitions: TransitionState) {
        let mut reverts = Vec::new();
        for (address, transition) in transitions.take().transitions.into_iter() {
            // add new contract if it was created/changed.
            if let Some((hash, new_bytecode)) = transition.has_new_contract() {
                self.contracts.insert(hash, new_bytecode.clone());
            }
            // update state and create revert.
            let revert = match self.state.entry(address) {
                hash_map::Entry::Occupied(mut entry) => {
                    let this_account = entry.get_mut();
                    // update and create revert if it is present
                    this_account.update_and_create_revert(transition)
                }
                hash_map::Entry::Vacant(entry) => {
                    // make revert from transition account
                    entry.insert(transition.present_bundle_account());
                    transition.create_revert()
                }
            };

            // append revert if present.
            if let Some(revert) = revert {
                reverts.push((address, revert));
            }
        }
        self.reverts.push(reverts);
    }

    /// Return plain state update
    pub fn take_plain_state(&mut self) -> HashMap<B160, BundleAccount> {
        core::mem::take(&mut self.state)
    }

    // Nuke the bundle state and return sorted plain state.
    pub fn take_sorted_plain_change(&mut self) -> StateChangeset {
        let mut accounts = Vec::new();
        let mut storage = Vec::new();

        for (address, account) in self.state.drain().into_iter() {
            // append account info if it is changed.
            if account.is_info_changed() {
                let mut info = account.info;
                info.as_mut().map(|a| a.code = None);
                accounts.push((address, info));
            }

            // append storage changes
            let mut account_storage_changed = Vec::with_capacity(account.storage.len());
            for (key, slot) in account.storage {
                if slot.is_changed() {
                    account_storage_changed.push((key, slot.present_value));
                }
            }

            account_storage_changed.sort_by(|a, b| a.0.cmp(&b.0));
            // append storage changes to account.
            storage.push((
                address,
                (account.status.was_destroyed(), account_storage_changed),
            ));
        }

        accounts.par_sort_unstable_by(|a, b| a.0.cmp(&b.0));
        storage.par_sort_unstable_by(|a, b| a.0.cmp(&b.0));

        let mut contracts = self.contracts.drain().into_iter().collect::<Vec<_>>();
        contracts.par_sort_unstable_by(|a, b| a.0.cmp(&b.0));

        StateChangeset {
            accounts,
            storage,
            contracts,
        }
    }

    pub fn take_reverts(&mut self) -> StateReverts {
        let mut state_reverts = StateReverts::default();
        for reverts in self.reverts.drain(..) {
            let mut accounts = Vec::new();
            let mut storage = Vec::new();
            for (address, revert_account) in reverts.into_iter() {
                match revert_account.account {
                    AccountInfoRevert::RevertTo(acc) => accounts.push((address, Some(acc))),
                    AccountInfoRevert::DeleteIt => accounts.push((address, None)),
                    AccountInfoRevert::DoNothing => (),
                }
                if !revert_account.storage.is_empty() {
                    let mut account_storage = Vec::new();
                    for (key, revert_slot) in revert_account.storage {
                        account_storage.push((key, revert_slot.to_previous_value()));
                    }
                    account_storage.par_sort_unstable_by(|a, b| a.0.cmp(&b.0));
                    storage.push((address, revert_account.wipe_storage, account_storage));
                }
            }
            accounts.par_sort_unstable_by(|a, b| a.0.cmp(&b.0));
            state_reverts.accounts.push(accounts);
            state_reverts.storage.push(storage);
        }

        state_reverts
    }
}
