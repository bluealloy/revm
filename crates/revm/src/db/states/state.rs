use core::convert::Infallible;

use super::{cache::CacheState, BundleAccount, BundleState};
use crate::{db::EmptyDB, BlockState};
use revm_interpreter::primitives::{
    db::{Database, DatabaseCommit},
    hash_map::Entry,
    Account, AccountInfo, Bytecode, HashMap, B160, B256, U256,
};

/// State of blockchain.
pub struct State<DBError> {
    /// Cached state contains both changed from evm executiong and cached/loaded account/storages
    /// from database. This allows us to have only one layer of cache where we can fetch data.
    /// Additionaly we can introuduce some preloading of data from database.
    pub cache: CacheState,
    /// Optional database that we use to fetch data from. If database is not present, we will
    /// return not existing account and storage.
    pub database: Box<dyn Database<Error = DBError>>,
    /// Build reverts and state that gets applied to the state.
    pub transition_builder: Option<TransitionBuilder>,
    /// Is state clear enabled
    /// TODO: should we do it as block number, it would be easier.
    pub has_state_clear: bool,
}

#[derive(Debug, Clone, Default)]
pub struct TransitionBuilder {
    /// Block state, it aggregates transactions transitions into one state.
    pub block_state: BlockState,
    /// After block is finishes we merge those changes inside bundle.
    /// Bundle is used to update database and create changesets.
    pub bundle_state: BundleState,
}

/// For State that does not have database.
impl State<Infallible> {
    pub fn new_cached() -> Self {
        Self {
            cache: CacheState::default(),
            database: Box::new(EmptyDB::default()),
            transition_builder: None,
            has_state_clear: true,
        }
    }

    pub fn new_cached_with_transition() -> Self {
        let db = Box::new(EmptyDB::default());
        Self {
            cache: CacheState::default(),
            database: db,
            transition_builder: Some(TransitionBuilder {
                block_state: BlockState::new(false),
                bundle_state: BundleState::default(),
            }),
            has_state_clear: true,
        }
    }

    pub fn new() -> Self {
        let db = Box::new(EmptyDB::default());
        Self {
            cache: CacheState::default(),
            database: db,
            transition_builder: None,
            has_state_clear: true,
        }
    }
}

impl<DBError> State<DBError> {
    /// State clear EIP-161 is enabled in Spurious Dragon hardfork.
    pub fn enable_state_clear_eip(&mut self) {
        self.has_state_clear = true;
        self.transition_builder
            .as_mut()
            .map(|t| t.block_state.set_state_clear());
    }

    pub fn new_with_transtion(db: Box<dyn Database<Error = DBError>>) -> Self {
        Self {
            cache: CacheState::default(),
            database: db,
            transition_builder: Some(TransitionBuilder {
                block_state: BlockState::new(false),
                bundle_state: BundleState::default(),
            }),
            has_state_clear: true,
        }
    }

    /// Insert account to cache.
    pub fn insert_account(&mut self, address: B160, account: AccountInfo) {
        //self.cache.accounts.insert(address, account);
    }

    // Insert storage to cache.
    pub fn insert_storage(&mut self, address: B160, index: U256, value: U256) {
        //self.cache.insert_storage(address, index, value);
    }
}

impl<DBError> Database for State<DBError> {
    type Error = DBError;

    fn basic(&mut self, address: B160) -> Result<Option<AccountInfo>, Self::Error> {
        // get from cache
        if let Some(account) = self.cache.accounts.get(&address) {
            return Ok(account.account_info());
        }

        self.database.basic(address)
    }

    fn code_by_hash(
        &mut self,
        code_hash: revm_interpreter::primitives::B256,
    ) -> Result<Bytecode, Self::Error> {
        self.database.code_by_hash(code_hash)
    }

    fn storage(&mut self, address: B160, index: U256) -> Result<U256, Self::Error> {
        // get from cache
        if let Some(account) = self.cache.accounts.get(&address) {
            return Ok(account.storage_slot(index).unwrap_or_default());
        }

        self.database.storage(address, index)
    }

    fn block_hash(&mut self, number: U256) -> Result<B256, Self::Error> {
        self.database.block_hash(number)
    }
}

impl<DB: Database> DatabaseCommit for State<DB> {
    fn commit(&mut self, evm_state: HashMap<B160, Account>) {
        //println!("PRINT STATE:");
        for (address, account) in evm_state {
            //println!("\n------:{:?} -> {:?}", address, account);
            if !account.is_touched() {
                continue;
            } else if account.is_selfdestructed() {
                // If it is marked as selfdestructed we to changed state to destroyed.
                match self.cache.accounts.entry(address) {
                    Entry::Occupied(mut entry) => {
                        let this = entry.get_mut();
                        this.selfdestruct();
                    }
                    Entry::Vacant(entry) => {
                        // if account is not present in db, we can just mark it as destroyed.
                        // This means that account was not loaded through this state.
                        entry.insert(BundleAccount::new_destroyed());
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
                match self.cache.accounts.entry(address) {
                    // if account is already present id db.
                    Entry::Occupied(mut entry) => {
                        let this = entry.get_mut();
                        this.newly_created(account.info.clone(), &account.storage)
                    }
                    Entry::Vacant(entry) => {
                        // This means that account was not loaded through this state.
                        // and we trust that account is empty.
                        entry.insert(BundleAccount::new_newly_created(
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
                    match self.cache.accounts.entry(address) {
                        Entry::Occupied(mut entry) => {
                            entry.get_mut().touch_empty();
                        }
                        Entry::Vacant(_entry) => {}
                    }
                    // else do nothing as account is not existing
                    continue;
                }

                // mark account as changed.
                match self.cache.accounts.entry(address) {
                    Entry::Occupied(mut entry) => {
                        let this = entry.get_mut();
                        this.change(account.info.clone(), account.storage.clone());
                    }
                    Entry::Vacant(entry) => {
                        // It is assumed initial state is Loaded
                        entry.insert(BundleAccount::new_changed(
                            account.info.clone(),
                            account.storage.clone(),
                        ));
                    }
                }
            }
        }
    }
}
