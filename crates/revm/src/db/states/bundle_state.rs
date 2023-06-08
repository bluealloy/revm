use super::{AccountRevert, BundleAccount, TransitionState};
use revm_interpreter::primitives::{hash_map, AccountInfo, Bytecode, HashMap, B160, B256, U256};

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
    pub fn take_sorted_plain_change(&mut self) -> PlainStateChange {
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
            // append storage changes to account.
            storage.push((address, account_storage_changed));
        }

        PlainStateChange {
            accounts,
            storage,
            contracts: self.contracts.drain().into_iter().collect::<Vec<_>>(),
        }
    }

    pub fn take_reverts(&mut self) -> Vec<Vec<(B160, AccountRevert)>> {
        core::mem::take(&mut self.reverts)
    }
}

/// Sorted accounts/storages/contracts for inclusion into database.
/// Structure is made so it is easier to apply dirrectly to database
/// that mostly have saparate tables to store account/storage/contract data.
#[derive(Clone, Debug, Default)]
pub struct PlainStateChange {
    /// Vector of account presorted by address, with removed contracts bytecode
    pub accounts: Vec<(B160, Option<AccountInfo>)>,
    /// Vector of storage presorted by address
    pub storage: Vec<(B160, Vec<(U256, U256)>)>,
    /// Vector of contracts presorted by bytecode hash
    pub contracts: Vec<(B256, Bytecode)>,
}

pub struct PlainRevert {
    /// Vector of account presorted by anddress, with removed cotracts bytecode
    ///
    /// Note: AccountInfo None means that account needs to be removed.
    pub accounts: Vec<(B160, Option<AccountInfo>)>,
    /// Vector of storage presorted by address
    /// U256::ZERO means that storage needs to be removed.
    pub storage: Vec<(B160, Vec<(U256, U256)>)>,
    /// Vector of contracts presorted by bytecode hash
    ///
    /// TODO: u64 counter is still not used. but represent number of times this contract was
    /// created, as multiple accounts can create same contract bytes.
    pub contracts: Vec<(B256, (u64, Bytecode))>,
}
