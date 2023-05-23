use revm_interpreter::primitives::{
    db::{Database, DatabaseCommit},
    hash_map::Entry,
    AccountInfo, Bytecode, Account, HashMap, State, StorageSlot, B160, B256, U256,
};

use super::{block_account::BlockAccount, PlainAccount};

/// TODO Rename this to become StorageWithOriginalValues or something like that.
/// This is used inside EVM and for block state. It is needed for block state to
/// be able to create changeset agains bundle state.
///
/// This storage represent values that are before block changed.
///
/// Note: Storage that we get EVM contains original values before t
pub type Storage = HashMap<U256, StorageSlot>;

#[derive(Clone, Debug, Default)]
pub struct BlockState {
    /// Block state account with account state
    pub accounts: HashMap<B160, BlockAccount>,
    /// created contracts
    pub contracts: HashMap<B256, Bytecode>,
    /// Has EIP-161 state clear enabled (Spurious Dragon hardfork).
    pub has_state_clear: bool,
}

impl DatabaseCommit for BlockState {
    fn commit(&mut self, changes: HashMap<B160, Account>) {
        self.apply_evm_state(&changes)
    }
}

impl BlockState {
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            contracts: HashMap::new(),
            has_state_clear: true,
        }
    }
    /// Legacy without state clear flag enabled
    pub fn new_legacy() -> Self {
        Self {
            accounts: HashMap::new(),
            contracts: HashMap::new(),
            has_state_clear: false,
        }
    }
    /// Used for tests only. When transitioned it is not recoverable
    pub fn set_state_clear(&mut self) {
        if self.has_state_clear == true {
            return;
        }

        self.has_state_clear = true;
    }

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
            .insert(address, BlockAccount::new_loaded_not_existing());
    }

    pub fn insert_account(&mut self, address: B160, info: AccountInfo) {
        let account = if !info.is_empty() {
            BlockAccount::new_loaded(info, HashMap::default())
        } else {
            BlockAccount::new_loaded_empty_eip161(HashMap::default())
        };
        self.accounts.insert(address, account);
    }

    pub fn insert_account_with_storage(
        &mut self,
        address: B160,
        info: AccountInfo,
        storage: Storage,
    ) {
        let account = if !info.is_empty() {
            BlockAccount::new_loaded(info, storage)
        } else {
            BlockAccount::new_loaded_empty_eip161(storage)
        };
        self.accounts.insert(address, account);
    }

    pub fn apply_evm_state(&mut self, evm_state: &State) {
        //println!("PRINT STATE:");
        for (address, account) in evm_state {
            //println!("\n------:{:?} -> {:?}", address, account);
            if !account.is_touched() {
                continue;
            } else if account.is_selfdestructed() {
                // If it is marked as selfdestructed we to changed state to destroyed.
                match self.accounts.entry(*address) {
                    Entry::Occupied(mut entry) => {
                        let this = entry.get_mut();
                        this.selfdestruct();
                    }
                    Entry::Vacant(entry) => {
                        // if account is not present in db, we can just mark it as destroyed.
                        // This means that account was not loaded through this state.
                        entry.insert(BlockAccount::new_destroyed());
                    }
                }
                continue;
            }
            let is_empty = account.is_empty();
            if account.is_created() {
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
                match self.accounts.entry(*address) {
                    // if account is already present id db.
                    Entry::Occupied(mut entry) => {
                        let this = entry.get_mut();
                        this.newly_created(account.info.clone(), &account.storage)
                    }
                    Entry::Vacant(entry) => {
                        // This means that account was not loaded through this state.
                        // and we trust that account is empty.
                        entry.insert(BlockAccount::new_newly_created(
                            account.info.clone(),
                            account.storage.clone(),
                        ));
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
                    match self.accounts.entry(*address) {
                        Entry::Occupied(mut entry) => {
                            entry.get_mut().touch_empty();
                        }
                        Entry::Vacant(_entry) => {}
                    }
                    // else do nothing as account is not existing
                    continue;
                }

                // mark account as changed.
                match self.accounts.entry(*address) {
                    Entry::Occupied(mut entry) => {
                        let this = entry.get_mut();
                        this.change(account.info.clone(), account.storage.clone());
                    }
                    Entry::Vacant(entry) => {
                        // It is assumed initial state is Loaded
                        entry.insert(BlockAccount::new_changed(
                            account.info.clone(),
                            account.storage.clone(),
                        ));
                    }
                }
            }
        }
    }
}

impl Database for BlockState {
    type Error = ();

    fn basic(&mut self, address: B160) -> Result<Option<AccountInfo>, Self::Error> {
        if let Some(account) = self.accounts.get(&address) {
            return Ok(account.account_info());
        }

        Ok(None)
    }

    fn code_by_hash(
        &mut self,
        _code_hash: revm_interpreter::primitives::B256,
    ) -> Result<Bytecode, Self::Error> {
        unreachable!("Code is always returned in basic account info")
    }

    fn storage(&mut self, address: B160, index: U256) -> Result<U256, Self::Error> {
        if let Some(account) = self.accounts.get(&address) {
            return Ok(account.storage_slot(index).unwrap_or_default());
        }

        Ok(U256::ZERO)
    }

    fn block_hash(&mut self, _number: U256) -> Result<B256, Self::Error> {
        Ok(B256::zero())
    }
}
