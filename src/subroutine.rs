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

pub struct SubRoutine {
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
    /// is static subroutine. SSTORE and various CALLs are not permited
    is_static: bool,
}

/// SubRoutine checkpoint that will help us to go back from this
pub struct SubRoutineCheckpoint {
    log_i: usize,
    changeset_i: usize,
    depth: usize,
}

impl SubRoutine {
    pub fn new() -> SubRoutine {
        let mut changeset = Vec::new();
        changeset.push(HashMap::new());
        Self {
            state: HashMap::new(),
            precompiles: HashSet::new(),
            logs: Vec::new(),
            changeset,
            depth: 0,
            is_static: false,
        }
    }

    pub fn modified_states(&mut self) {
        //TODO return all accounts and storages that are modified by sstore/set_balance/inc_nonce/create_acc
    }

    pub fn is_static(&mut self) -> bool {
        self.is_static
    }

    pub fn set_static(&mut self, is_static: bool) {
        self.is_static = is_static;
    }

    /// load account into memory. return if it is cold or hot accessed
    pub fn load_account<DB: Database>(&mut self, address: H160, db: &mut DB) -> (&Account, bool) {
        let is_cold = match self.state.entry(address.clone()) {
            Entry::Occupied(occ) => false,
            Entry::Vacant(vac) => {
                let acc: Account = db.basic(address.clone()).into();
                vac.insert(acc.clone());
                // insert none in changeset that represent that we just loaded this acc in this subroutine
                self.changeset
                    .last_mut()
                    .unwrap()
                    .insert(address.clone(), None);
                true
            }
        };
        (self.state.get(&address).unwrap(), is_cold)
    }

    pub fn load_code<DB: Database>(&mut self, address: H160, db: &mut DB) -> (&Account, bool) {
        let (_, is_cold) = self.load_account(address.clone(), db);
        let acc = self.state.get_mut(&address).unwrap();

        if acc.info.code.is_none() {
            let code = if let Some(code_hash) = acc.info.code_hash {
                db.code_by_hash(code_hash)
            } else {
                db.code(address)
            };
            acc.info.code = Some(code);
        }
        (acc, is_cold)
    }

    /// Use it with load_account function.
    pub fn account(&self, address: H160) -> &Account {
        self.state.get(&address).unwrap() // Allways assume that acc is already loaded
    }

    /// use it only if you know that acc is hot
    pub fn account_mut(&mut self, address: H160) -> &mut Account {
        self.state.get_mut(&address).unwrap()
    }

    pub fn depth(&self) -> usize {
        self.depth
    }

    pub fn inc_nonce(&mut self, address: H160) {
        // asume account is hot and loaded
        let acc = self.state.get_mut(&address).unwrap();
        if let Entry::Vacant(entry) = self.changeset.last_mut().unwrap().entry(address) {
            // insert unchanged account version
            entry.insert(Some((acc.clone(), HashSet::new())));
        }
        acc.info.nonce += 1;
    }

    pub fn transfer<DB: Database>(
        &mut self,
        from: H160,
        to: H160,
        value: U256,
        db: &mut DB,
    ) -> Result<(bool, bool), ExitError> {
        // load accounts
        let (_, from_is_cold) = self.load_account(from, db);
        let (_, to_is_cold) = self.load_account(to, db);
        // check from balance and substract value
        let from = self.state.get_mut(&from).unwrap();
        if from.info.balance < value {
            return Err(ExitError::OutOfFund);
        }
        from.info.balance -= value;
        // add value to to account
        let to = self.state.get_mut(&to).unwrap();
        to.info.balance += value;
        Ok((from_is_cold, to_is_cold))
    }

    pub fn create_checkpoint(&mut self) -> SubRoutineCheckpoint {
        self.depth += 1;
        let checkpoint = SubRoutineCheckpoint {
            log_i: self.logs.len(),
            changeset_i: self.changeset.len(),
            depth: self.depth,
        };
        self.changeset.push(Default::default());
        checkpoint
    }

    ///
    pub fn checkpoint_commit(
        &mut self,
        _checkpoint: SubRoutineCheckpoint,
    ) {
        self.depth -= 1;
        // we are continuing to use present checkpoint because it is merge between ours and parents
    }

    pub fn checkpoint_revert(&mut self, checkpoint: SubRoutineCheckpoint) {
        self.logs.truncate(checkpoint.log_i);
        // TODO revert all changes from changeset
        self.changeset.truncate(checkpoint.changeset_i);
        self.depth -= 1;
    }

    pub fn checkpoint_discard(&mut self, checkpoint: SubRoutineCheckpoint) {
        self.logs.truncate(checkpoint.log_i);
        // TODO revert all changes from changeset
        self.changeset.truncate(checkpoint.changeset_i);
        self.depth -= 1;
    }

    // CHECK probably expand it to target address where value is going to be transfered
    pub fn destroy_storage(&mut self, address: H160) {
        let acc = self.state.get_mut(&address).unwrap();
        // if there is no account in this subroutine changeset,
        // save it so that it can be restored if subroutine needs to be rediscarded.
        self.changeset
            .last_mut()
            .unwrap()
            .entry(address.clone())
            .or_insert(Some((acc.clone(), HashSet::new())));
        acc.destroyed = true;
        acc.storage.clear();
    }

    // CHECK account is allways present for storage that we want to access
    pub fn sload<DB: Database>(&mut self, address: H160, index: H256, db: &mut DB) -> (H256, bool) {
        let acc = self.state.get_mut(&address).unwrap(); // asume acc is hot
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

    // CHECK, but for now it seems just fine! TODO check destroyed flah
    pub fn sstore<DB: Database>(
        &mut self,
        address: H160,
        index: H256,
        value: H256,
        db: &mut DB,
    ) -> (H256, bool) {
        // assume that acc exists
        let (present, is_cold) = self.sload(address, index, db);
        if present == value {
            return (present, is_cold);
        }
        let acc = self.state.get_mut(&address).unwrap();
        // insert original value inside changeset
        match self.changeset.last_mut().unwrap().entry(address.clone()) {
            // there is account present inside changeset.
            Entry::Occupied(mut occ) => {
                if let Some((acc, cold_storage)) = occ.get_mut() {
                    if is_cold {
                        cold_storage.insert(index);
                    } else {
                        // insert original value inside set if there is nothing there.
                        acc.storage.entry(index).or_insert(present);
                    }
                }
                // if changeset is empty account is cold loaded inside this subroutine and we dont need to do anything
            }
            // if account is not present that means that account is loaded in past and we yet didnt made any
            // changes inside this subroutine. Make a copy of curreny not changed acc and set it inside changeset
            // not sure if this is possible because contract account is loaded before it is called
            Entry::Vacant(vac) => {
                let mut was_cold_storage = HashSet::new();
                if is_cold {
                    was_cold_storage.insert(index);
                }
                vac.insert(Some((acc.clone(), was_cold_storage)));
            }
        }
        // insert value into present state.
        acc.storage.insert(index, value);

        (present, is_cold)
    }

    /// push log into subroutine
    pub fn log(&mut self, log: Log) {
        self.logs.push(log);
    }
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
