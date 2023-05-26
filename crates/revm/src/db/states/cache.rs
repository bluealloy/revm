use super::{
    plain_account::PlainStorage, transition_account::TransitionAccount, BundleAccount, PlainAccount,
};
use revm_interpreter::primitives::{
    hash_map::Entry, AccountInfo, Bytecode, HashMap, State as EVMState, B160, B256,
};

/// Cache state contains both modified and original values.
///
/// TODO add prunning of LRU read accounts. As we would like to keep only modifed data.
///
/// Sharading data between bundle execution can be done with help of bundle id.
/// That should help with unmarking account of old bundle and allowing them to be removed.
#[derive(Debug, Default)]
pub struct CacheState {
    /// Block state account with account state
    pub accounts: HashMap<B160, BundleAccount>,
    /// created contracts
    /// TODO add bytecode counter for number of bytecodes added/removed.
    pub contracts: HashMap<B256, Bytecode>,
    /// Has EIP-161 state clear enabled (Spurious Dragon hardfork).
    pub has_state_clear: bool,
}

impl CacheState {
    pub fn trie_account(&self) -> impl IntoIterator<Item = (B160, &PlainAccount)> {
        self.accounts.iter().filter_map(|(address, account)| {
            account
                .account
                .as_ref()
                .map(|plain_acc| (*address, plain_acc))
        })
    }

    pub fn insert_not_existing(&mut self, address: B160) {
        self.accounts
            .insert(address, BundleAccount::new_loaded_not_existing());
    }

    pub fn insert_account(&mut self, address: B160, info: AccountInfo) {
        let account = if !info.is_empty() {
            BundleAccount::new_loaded(info, HashMap::default())
        } else {
            BundleAccount::new_loaded_empty_eip161(HashMap::default())
        };
        self.accounts.insert(address, account);
    }

    pub fn insert_account_with_storage(
        &mut self,
        address: B160,
        info: AccountInfo,
        storage: PlainStorage,
    ) {
        let account = if !info.is_empty() {
            BundleAccount::new_loaded(info, storage)
        } else {
            BundleAccount::new_loaded_empty_eip161(storage)
        };
        self.accounts.insert(address, account);
    }

    /// Make transitions.
    pub fn apply_evm_state(&mut self, evm_state: EVMState) -> Vec<(B160, TransitionAccount)> {
        let mut transitions = Vec::new();
        //println!("PRINT STATE:");
        for (address, account) in evm_state {
            //println!("\n------:{:?} -> {:?}", address, account);

            // TODO move plain_storage to place where it is needed
            let plain_storage = account
                .storage
                .iter()
                .map(|(k, v)| (*k, v.present_value))
                .collect();
            if !account.is_touched() {
                // not touched account are never changed.
                continue;
            } else if account.is_selfdestructed() {
                // If it is marked as selfdestructed we to changed state to destroyed.
                let transition = match self.accounts.entry(address) {
                    Entry::Occupied(mut entry) => {
                        let this = entry.get_mut();
                        this.selfdestruct()
                    }
                    Entry::Vacant(entry) => {
                        // if account is not present in db, we can just mark it as destroyed.
                        // This means that account was not loaded through this state.
                        entry.insert(BundleAccount::new_destroyed());
                        // TODO
                        TransitionAccount::default()
                    }
                };
                transitions.push((address, transition));
                continue;
            }

            let is_empty = account.is_empty();
            let transition = if account.is_created() {
                // Note: it can happen that created contract get selfdestructed in same block
                // that is why is newly created is checked after selfdestructed
                //
                // Note: Create2 (Petersburg) was after state clear EIP (Spurious Dragon)
                // so we dont need to clear
                //
                // Note: It is possibility to create KECCAK_EMPTY contract with some storage
                // by just setting storage inside CRATE contstructor. Overlap of those contracts
                // is not possible because CREATE2 is introduced later.
                //
                match self.accounts.entry(address) {
                    // if account is already present id db.
                    Entry::Occupied(mut entry) => {
                        let this = entry.get_mut();
                        this.newly_created(account.info, account.storage)
                    }
                    Entry::Vacant(entry) => {
                        // This means that account was not loaded through this state.
                        // and we trust that account is empty.
                        entry.insert(BundleAccount::new_newly_created(
                            account.info.clone(),
                            plain_storage,
                        ));
                        // TODO
                        TransitionAccount::default()
                    }
                }
            } else {
                // Account is touched, but not selfdestructed or newly created.
                // Account can be touched and not changed.

                // And when empty account is touched it needs to be removed from database.
                // EIP-161 state clear
                if self.has_state_clear && is_empty {
                    // TODO Check if sending ZERO value created account pre state clear???

                    // touch empty account.
                    match self.accounts.entry(address) {
                        Entry::Occupied(mut entry) => {
                            entry.get_mut().touch_empty();
                        }
                        Entry::Vacant(_entry) => {}
                    }
                    // else do nothing as account is not existing

                    // TODO do transition
                    continue;
                }

                // mark account as changed.
                match self.accounts.entry(address) {
                    Entry::Occupied(mut entry) => {
                        let this = entry.get_mut();
                        this.change(account.info, account.storage)
                    }
                    Entry::Vacant(entry) => {
                        // It is assumed initial state is Loaded
                        entry.insert(BundleAccount::new_changed(
                            account.info.clone(),
                            plain_storage,
                        ));
                        // TODO
                        TransitionAccount::default()
                    }
                }
            };
            transitions.push((address, transition));
        }
        transitions
    }
}
