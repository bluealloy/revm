use super::{
    plain_account::PlainStorage, transition_account::TransitionAccount, CacheAccount, PlainAccount,
};
use bytecode::Bytecode;
use primitives::{Address, HashMap, B256};
use state::{Account, AccountInfo, EvmState};
use std::vec::Vec;

/// Cache state contains both modified and original values
///
/// # Note
/// Cache state is main state that revm uses to access state.
///
/// It loads all accounts from database and applies revm output to it.
///
/// It generates transitions that is used to build BundleState.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CacheState {
    /// Block state account with account state
    pub accounts: HashMap<Address, CacheAccount>,
    /// Created contracts
    pub contracts: HashMap<B256, Bytecode>,
    /// Has EIP-161 state clear enabled (Spurious Dragon hardfork)
    pub has_state_clear: bool,
}

impl Default for CacheState {
    fn default() -> Self {
        Self::new(true)
    }
}

impl CacheState {
    /// Creates a new default state.
    pub fn new(has_state_clear: bool) -> Self {
        Self {
            accounts: HashMap::default(),
            contracts: HashMap::default(),
            has_state_clear,
        }
    }

    /// Sets state clear flag. EIP-161.
    pub fn set_state_clear_flag(&mut self, has_state_clear: bool) {
        self.has_state_clear = has_state_clear;
    }

    /// Helper function that returns all accounts.
    ///
    /// Used inside tests to generate merkle tree.
    pub fn trie_account(&self) -> impl IntoIterator<Item = (Address, &PlainAccount)> {
        self.accounts.iter().filter_map(|(address, account)| {
            account
                .account
                .as_ref()
                .map(|plain_acc| (*address, plain_acc))
        })
    }

    /// Inserts not existing account.
    pub fn insert_not_existing(&mut self, address: Address) {
        self.accounts
            .insert(address, CacheAccount::new_loaded_not_existing());
    }

    /// Inserts Loaded (Or LoadedEmptyEip161 if account is empty) account.
    pub fn insert_account(&mut self, address: Address, info: AccountInfo) {
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
        address: Address,
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

    /// Applies output of revm execution and create account transitions that are used to build BundleState.
    pub fn apply_evm_state(&mut self, evm_state: EvmState) -> Vec<(Address, TransitionAccount)> {
        let mut transitions = Vec::with_capacity(evm_state.len());
        for (address, account) in evm_state {
            if let Some(transition) = self.apply_account_state(address, account) {
                transitions.push((address, transition));
            }
        }
        transitions
    }

    /// Pretty print the cache state for debugging purposes.
    pub fn pretty_print(&self) -> String {
        let mut output = String::new();
        output.push_str("CacheState:\n");
        output.push_str(&format!(
            "  (state_clear_enabled: {}, ",
            self.has_state_clear
        ));
        output.push_str(&format!("accounts: {} total)\n", self.accounts.len()));

        // Sort accounts by address for consistent output
        let mut accounts: Vec<_> = self.accounts.iter().collect();
        accounts.sort_by_key(|(addr, _)| *addr);

        let mut contracts = self.contracts.clone();

        for (address, account) in accounts {
            output.push_str(&format!("  [{address}]:\n"));
            output.push_str(&format!("    status: {:?}\n", account.status));

            if let Some(plain_account) = &account.account {
                let code_hash = plain_account.info.code_hash;
                output.push_str(&format!("    balance: {}\n", plain_account.info.balance));
                output.push_str(&format!("    nonce: {}\n", plain_account.info.nonce));
                output.push_str(&format!("    code_hash: {code_hash}\n"));

                if let Some(code) = &plain_account.info.code {
                    if !code.is_empty() {
                        contracts.insert(code_hash, code.clone());
                    }
                }

                if !plain_account.storage.is_empty() {
                    output.push_str(&format!(
                        "    storage: {} slots\n",
                        plain_account.storage.len()
                    ));
                    // Sort storage by key for consistent output
                    let mut storage: Vec<_> = plain_account.storage.iter().collect();
                    storage.sort_by_key(|(key, _)| *key);

                    for (key, value) in storage.iter() {
                        output.push_str(&format!("      [{key:#x}]: {value:#x}\n"));
                    }
                }
            } else {
                output.push_str("    account: None (destroyed or non-existent)\n");
            }
        }

        if !contracts.is_empty() {
            output.push_str(&format!("  contracts: {} total\n", contracts.len()));
            for (hash, bytecode) in contracts.iter() {
                let len = bytecode.len();
                output.push_str(&format!("    [{hash}]: {len} bytes\n"));
            }
        }

        output.push_str("}\n");
        output
    }

    /// Applies updated account state to the cached account.
    ///
    /// Returns account transition if applicable.
    fn apply_account_state(
        &mut self,
        address: Address,
        account: Account,
    ) -> Option<TransitionAccount> {
        // Not touched account are never changed.
        if !account.is_touched() {
            return None;
        }

        let this_account = self
            .accounts
            .get_mut(&address)
            .expect("All accounts should be present inside cache");

        // If it is marked as selfdestructed inside revm
        // we need to changed state to destroyed.
        if account.is_selfdestructed() {
            return this_account.selfdestruct();
        }

        let is_created = account.is_created();
        let is_empty = account.is_empty();

        // Transform evm storage to storage with previous value.
        let changed_storage = account
            .storage
            .into_iter()
            .filter(|(_, slot)| slot.is_changed())
            .map(|(key, slot)| (key, slot.into()))
            .collect();

        // Note: It can happen that created contract get selfdestructed in same block
        // that is why is_created is checked after selfdestructed
        //
        // Note: Create2 opcode (Petersburg) was after state clear EIP (Spurious Dragon)
        //
        // Note: It is possibility to create KECCAK_EMPTY contract with some storage
        // by just setting storage inside CRATE constructor. Overlap of those contracts
        // is not possible because CREATE2 is introduced later.
        if is_created {
            return Some(this_account.newly_created(account.info, changed_storage));
        }

        // Account is touched, but not selfdestructed or newly created.
        // Account can be touched and not changed.
        // And when empty account is touched it needs to be removed from database.
        // EIP-161 state clear
        if is_empty {
            if self.has_state_clear {
                // Touch empty account.
                this_account.touch_empty_eip161()
            } else {
                // If account is empty and state clear is not enabled we should save
                // empty account.
                this_account.touch_create_pre_eip161(changed_storage)
            }
        } else {
            Some(this_account.change(account.info, changed_storage))
        }
    }
}
