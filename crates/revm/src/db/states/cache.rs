use super::{
    plain_account::PlainStorage, transition_account::TransitionAccount, CacheAccount, PlainAccount,
};
use revm_interpreter::primitives::{
    hash_map::Entry, AccountInfo, Bytecode, HashMap, State as EVMState, B160, B256,
};

/// Cache state contains both modified and original values.
///
/// Cache state is main state that revm uses to access state.
/// It loads all accounts from database and applies revm output to it.
///
/// It generates transitions that is used to build BundleState.
#[derive(Debug, Clone)]
pub struct CacheState {
    /// Block state account with account state
    pub accounts: HashMap<B160, CacheAccount>,
    /// created contracts
    /// TODO add bytecode counter for number of bytecodes added/removed.
    pub contracts: HashMap<B256, Bytecode>,
    /// Has EIP-161 state clear enabled (Spurious Dragon hardfork).
    pub has_state_clear: bool,
}

impl Default for CacheState {
    fn default() -> Self {
        Self::new(true)
    }
}

impl CacheState {
    /// New default state.
    pub fn new(has_state_clear: bool) -> Self {
        Self {
            accounts: HashMap::default(),
            contracts: HashMap::default(),
            has_state_clear,
        }
    }

    /// Set state clear flag. EIP-161.
    pub fn set_state_clear_flag(&mut self, has_state_clear: bool) {
        self.has_state_clear = has_state_clear;
    }

    /// Helper function that returns all accounts.
    /// Used inside tests to generate merkle tree.
    pub fn trie_account(&self) -> impl IntoIterator<Item = (B160, &PlainAccount)> {
        self.accounts.iter().filter_map(|(address, account)| {
            account
                .account
                .as_ref()
                .map(|plain_acc| (*address, plain_acc))
        })
    }

    /// Insert not existing account.
    pub fn insert_not_existing(&mut self, address: B160) {
        self.accounts
            .insert(address, CacheAccount::new_loaded_not_existing());
    }

    /// Insert Loaded (Or LoadedEmptyEip161 if account is empty) account.
    pub fn insert_account(&mut self, address: B160, info: AccountInfo) {
        let account = if !info.is_empty() {
            CacheAccount::new_loaded(info, HashMap::default())
        } else {
            CacheAccount::new_loaded_empty_eip161(HashMap::default())
        };
        self.accounts.insert(address, account);
    }

    /// Similar to `insert_account` but with storage.
    pub fn insert_account_with_storage(
        &mut self,
        address: B160,
        info: AccountInfo,
        storage: PlainStorage,
    ) {
        let account = if !info.is_empty() {
            CacheAccount::new_loaded(info, storage)
        } else {
            CacheAccount::new_loaded_empty_eip161(storage)
        };
        self.accounts.insert(address, account);
    }

    /// Apply output of revm execution and create TransactionAccount
    /// that is used to build BundleState.
    pub fn apply_evm_state(&mut self, evm_state: EVMState) -> Vec<(B160, TransitionAccount)> {
        let mut transitions = Vec::with_capacity(evm_state.len());
        for (address, account) in evm_state {
            if !account.is_touched() {
                // not touched account are never changed.
                continue;
            } else if account.is_selfdestructed() {
                // If it is marked as selfdestructed inside revm
                // we need to changed state to destroyed.
                match self.accounts.entry(address) {
                    Entry::Occupied(mut entry) => {
                        let this = entry.get_mut();
                        if let Some(transition) = this.selfdestruct() {
                            transitions.push((address, transition));
                        }
                    }
                    Entry::Vacant(_entry) => {
                        // if account is not present in db, we can just mark it sa NotExisting.
                        // This should not happen as all account should be loaded through this state.
                        unreachable!("All account should be loaded from cache for selfdestruct");
                    }
                };
                continue;
            }
            let is_empty = account.is_empty();
            if account.is_created() {
                // Note: it can happen that created contract get selfdestructed in same block
                // that is why is_created is checked after selfdestructed
                //
                // Note: Create2 opcode (Petersburg) was after state clear EIP (Spurious Dragon)
                //
                // Note: It is possibility to create KECCAK_EMPTY contract with some storage
                // by just setting storage inside CRATE contstructor. Overlap of those contracts
                // is not possible because CREATE2 is introduced later.
                //
                match self.accounts.entry(address) {
                    // if account is already present id db.
                    Entry::Occupied(mut entry) => {
                        let this = entry.get_mut();
                        transitions
                            .push((address, this.newly_created(account.info, account.storage)))
                    }
                    Entry::Vacant(_entry) => {
                        // This means shold not happen as all accounts should be loaded through
                        // this state.
                        unreachable!("All account should be loaded from cache for created account");
                    }
                }
            } else {
                // Account is touched, but not selfdestructed or newly created.
                // Account can be touched and not changed.

                // And when empty account is touched it needs to be removed from database.
                // EIP-161 state clear
                if is_empty {
                    if self.has_state_clear {
                        // touch empty account.
                        match self.accounts.entry(address) {
                            Entry::Occupied(mut entry) => {
                                if let Some(transition) = entry.get_mut().touch_empty() {
                                    transitions.push((address, transition));
                                }
                            }
                            Entry::Vacant(_entry) => {
                                unreachable!("Empty account should be loaded in cache")
                            }
                        }
                    } else {
                        // if account is empty and state clear is not enabled we should save
                        // empty account.
                        match self.accounts.entry(address) {
                            Entry::Occupied(mut entry) => {
                                let transition =
                                    entry.get_mut().touch_create_eip161(account.storage);
                                transitions.push((address, transition));
                            }
                            Entry::Vacant(_entry) => {
                                unreachable!("Empty Account should be loaded in cache")
                            }
                        }
                    }
                    continue;
                }

                // mark account as changed.
                match self.accounts.entry(address) {
                    Entry::Occupied(mut entry) => {
                        let this = entry.get_mut();
                        // make a change and create transition.
                        transitions.push((address, this.change(account.info, account.storage)));
                    }
                    Entry::Vacant(_entry) => {
                        // It is assumed initial state is Loaded. Should not happen.
                        unreachable!("All account should be loaded from cache for change account");
                    }
                }
            };
        }
        transitions
    }
}
