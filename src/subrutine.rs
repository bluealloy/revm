use std::{
    collections::{btree_map::OccupiedEntry, hash_map::Entry, HashMap, HashSet},
    fs::File,
    ops::DerefMut,
    rc::Rc,
    thread::AccessError,
};

use bytes::Bytes;
use primitive_types::{H160, H256, U256};

use crate::{db::Database, error::ExitError, AccountInfo, Log, Transfer};

pub struct SubRutine {
    /// Applied changes to our state
    state: HashMap<H160, Account>,
    precompiles: HashSet<H160>,
    logs: Vec<Log>,
    /// It contains original values before they were changes.
    /// it is made like this so that we can revert to previous state in case of
    /// exit or revert
    /// if account is none it means that account was cold in previoud changeset
    /// Additional HashSet represent cold storage slots.
    changeset: Vec<HashMap<H160, Option<(Account, HashSet<H256>)>>>,
    /// how deep are we in call stack.
    depth: usize,
}

/// SubRutine checkpoint that will help us to go back from this
pub struct SubRutineCheckpoint {
    log_i: usize,
    changeset_i: usize,
    depth: usize,
}

impl SubRutine {
    pub fn new() -> SubRutine {
        let mut changeset = Vec::new();
        changeset.push(HashMap::new());
        Self {
            state: HashMap::new(),
            precompiles: HashSet::new(),
            logs: Vec::new(),
            changeset,
            depth: 0,
        }
    }

    /// load account into memory. return if it is cold or hot accessed
    pub fn load_account<DB: Database>(&mut self, address: H160, db: &mut DB) -> bool {
        let is_cold = match self.state.entry(address.clone()) {
            Entry::Occupied(occ) => false,
            Entry::Vacant(vac) => {
                let acc: Account = db.basic(address.clone()).into();
                vac.insert(acc.clone());
                // insert none in changeset that represent that we just loaded this acc in this subrutine
                self.changeset
                    .last_mut()
                    .unwrap()
                    .insert(address.clone(), None);
                true
            }
        };
        is_cold
    }

    pub fn load_code<DB: Database>(&mut self, address: H160, db: &mut DB) -> bool {
        let is_cold = self.load_account(address.clone(), db);
        let acc = self.state.get_mut(&address).unwrap();

        if acc.info.code.is_none() {
            let code = if let Some(code_hash) = acc.info.code_hash {
                db.code_by_hash(code_hash)
            } else {
                db.code(address)
            };
            acc.info.code = Some(code);
        }
        is_cold
    }
    // account is allways present for storage that we want to access
    pub fn load_storage<DB: Database>(
        &mut self,
        address: H160,
        index: H256,
        db: &mut DB,
    ) -> (H256, bool) {
        let acc = self.account_mut(address);
        match acc.storage.entry(index) {
            Entry::Occupied(occ) => (occ.get().clone(), false),
            Entry::Vacant(vac) => {
                if acc.destroyed {
                    (vac.insert(H256::zero()).clone(), true)
                } else {
                    (vac.insert(db.storage(address, index)).clone(), true)
                }
            }
        }
    }

    /// Use it with load_account function.
    pub fn account_mut(&mut self, address: H160) -> &mut Account {
        self.state.get_mut(&address).unwrap() // Allways assume that acc is already loaded
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn inc_nonce(&mut self, address: H160) {
        //TODO self.account_mut(address, backend).basic.nonce += U256::one();
    }

    pub fn transfer(&mut self, from: H160, to: H160, value: U256) -> Result<(), ExitError> {
        // TODO check funds and do transfer
        Ok(())
    }

    pub fn create_checkpoint(&mut self) -> SubRutineCheckpoint {
        self.depth += 1;
        let checkpoint = SubRutineCheckpoint {
            log_i: self.logs.len(),
            changeset_i: self.changeset.len(),
            depth: self.depth,
        };
        self.changeset.push(Default::default());
        checkpoint
    }

    pub fn exit_commit(&mut self, checkpoint: SubRutineCheckpoint) -> Result<(), ExitError> {
        self.depth -= 1;
        Ok(())
    }

    pub fn exit_revert(&mut self, checkpoint: SubRutineCheckpoint) {
        self.logs.truncate(checkpoint.log_i);
        // revert all changes from changeset
        self.changeset.truncate(checkpoint.changeset_i);
        self.depth -= 1;
    }

    pub fn exit_discard(&mut self, checkpoint: SubRutineCheckpoint) {
        self.logs.truncate(checkpoint.log_i);
        // revert all changes from changeset
        self.changeset.truncate(checkpoint.changeset_i);
        self.depth -= 1;
    }


    // TODO check, but for now it seems just fine! TODO check destroyed flah
    pub fn sstore<DB: Database>(
        &mut self,
        address: H160,
        index: H256,
        value: H256,
        db: &mut DB,
    ) -> (H256, bool) {
        // assume that acc exists
        let (present, is_cold) = self.load_storage(address, index, db);
        if present == value {
            return (present, is_cold);
        }
        let acc = self.state.get_mut(&address).unwrap();
        // insert original value inside changeset
        match self.changeset.last_mut().unwrap().entry(address.clone()) {
            Entry::Occupied(mut occ) => {
                // there is account present inside changeset.
                if let Some((acc, cold_storage)) = occ.get_mut() {
                    if is_cold {
                        cold_storage.insert(index);
                    } else {
                        // insert original value inside set if there is nothing there.
                        acc.storage.entry(index).or_insert(present);
                    }
                }
                // if changeset is empty account is cold loaded inside this subrutine and we dont need to do anything
            }
            Entry::Vacant(vac) => {
                // if account is not present that means that account is loaded in past and we yet didnt made any
                // changes inside this subrutine. Make a copy of curreny not changed acc and set it inside changeset
                let mut was_cold_storage = HashSet::new();
                if is_cold {
                    was_cold_storage.insert(index);
                }
                vac.insert(Some((acc.clone(),was_cold_storage)));
            }
        }
        // insert slot to state.
        acc.storage.insert(index, value);

        (present, is_cold)
    }

    pub fn log(&mut self, log: Log) {
        self.logs.push(log);
    }

    pub fn known_account(&self, address: &H160) -> Option<&Account> {
        self.state.get(&address)
    }
    /*

    pub fn known_basic(&self, address: H160) -> Option<AccountInfo> {
        self.known_account(address).map(|acc| acc.basic.clone())
    }

    pub fn known_code(&self, address: H160) -> Option<Vec<u8>> {
        self.known_account(address).and_then(|acc| acc.code.clone())
    }

    pub fn known_empty(&self, address: H160) -> Option<bool> {
        if let Some(account) = self.known_account(address) {
            if account.basic.balance != U256::zero() {
                return Some(false);
            }

            if account.basic.nonce != U256::zero() {
                return Some(false);
            }

            if let Some(code) = &account.code {
                return Some(
                    account.basic.balance == U256::zero()
                        && account.basic.nonce == U256::zero()
                        && code.is_empty(),
                );
            }
        }

        None
    }

    pub fn known_storage(&self, address: H160, key: H256) -> Option<H256> {
        if let Some(value) = self.storages.get(&(address, key)) {
            return Some(*value);
        }

        if let Some(account) = self.accounts.get(&address) {
            if account.reset {
                return Some(H256::default());
            }
        }

        if let Some(parent) = self.parent.as_ref() {
            return parent.known_storage(address, key);
        }

        None
    }

    pub fn known_original_storage(&self, address: H160, key: H256) -> Option<H256> {
        if let Some(account) = self.accounts.get(&address) {
            if account.reset {
                return Some(H256::default());
            }
        }

        if let Some(parent) = self.parent.as_ref() {
            return parent.known_original_storage(address, key);
        }

        None
    }*/
}

#[derive(Debug, Clone)]
pub struct Account {
    /// Balance of the account.
    pub info: AccountInfo,
    /// storage cache
    pub storage: HashMap<H256, H256>,
    /// if selfdestruct opcode is set destroyed flag will be true. If true we dont need to fetch slot from DB.
    pub destroyed: bool,
}

impl From<AccountInfo> for Account {
    fn from(info: AccountInfo) -> Self {
        Self {
            info,
            storage: HashMap::new(),
            destroyed: false,
        }
    }
}
