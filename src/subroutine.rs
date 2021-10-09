use crate::{
    collection::{vec, vec::Vec, Entry, Map},
    evm::SelfDestructResult,
    ExitRevert,
};

use core::mem::{self};

use bytes::Bytes;
use primitive_types::{H160, H256, U256};
use rlp::Decodable;
use secp256k1::curve::Field;

use crate::{db::Database, error::ExitError, AccountInfo, Log};

pub struct SubRoutine {
    /// Applied changes to our state
    state: State,
    /// logs
    logs: Vec<Log>,
    /// It contains original values before they were changes.
    /// it is made like this so that we can revert to previous state in case of
    /// exit or revert
    /// if account is none it means that account was cold in previoud changelog
    /// Additional HashSet represent cold storage slots.
    changelog: Vec<Map<H160, ChangeLog>>,
    /// how deep are we in call stack.
    depth: usize,
}

// contains old account changes and cold sload
// If None that means that account is loaded in this changeset. And not exists in past
// HashSet<H256> contains slot changelog
// acc.storage contains previous slots that are changes in this changelog
// acc.filth contains previous change
//pub type ChangeLog = Map<H160, Option<(Account, Map<H256, SlotChangeLog>)>>;

#[derive(Debug, Clone)]
pub enum ChangeLog {
    ColdLoaded,
    Dirty(DirtyChangeLog),
    Destroyed(Account),
    /// TODO check if there is possibility for acc to have some balance before contract is created.
    /// if that is possible we need U256 to revert to old value.
    /// storage/code/code_hash/nonce had default value.
    /// filth can only be Clean or Dirty. Clean for original balance and dirty with empty HashMap for changed balance.
    Created(U256, Filth),
    // precompile old balance value and flag if it is
    PrecompileBalanceChange(U256, bool), //TODO add it to balance change
}

#[derive(Debug, Clone)]
pub struct DirtyChangeLog {
    // contains previous values of slot.
    // If it is cold loaded in this subrutine SlotChangeLog will be COLD.
    // if it is hot and it gets changes somewhare in child subroutine, SlotChangeLog will contain old value OriginalDirty,
    dirty_storage: Map<H256, SlotChangeLog>,
    // account info, when reverting just overrride state value.
    info: AccountInfo,
    was_clean: bool,
}

pub type State = Map<H160, Account>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlotChangeLog {
    Cold,
    OriginalDirty(H256),
}

/// SubRoutine checkpoint that will help us to go back from this
pub struct SubRoutineCheckpoint {
    log_i: usize,
    changelog_i: usize,
    depth: usize,
}

impl SubRoutine {
    pub fn new(precompiles: Map<H160, AccountInfo>) -> SubRoutine {
        let state = precompiles
            .into_iter()
            .map(|(k, value)| {
                let mut acc = Account::from(value);
                acc.filth = Filth::Precompile(false);
                (k, acc)
            })
            .collect();

        Self {
            state,
            logs: Vec::new(),
            changelog: vec![Map::new(); 1],
            depth: 0,
        }
    }

    /// do cleanup and return modified state
    /// Do take that filthy stuff and return it back.
    /// Some states of Filth enum are internaly used but in output it can only be:
    /// 1. Dirty with empty Map (Map is internaly used). Only changed slots are returned in `storage` or
    /// 2. Destroyed if selfdestruct was called.
    pub fn finalize(&mut self) -> State {
        let mut out = Map::new();
        let state = mem::take(&mut self.state);
        for (add, mut acc) in state.into_iter() {
            let dirty = acc.filth.clean();
            match acc.filth {
                Filth::Destroyed | Filth::NewlyCreated | Filth::Precompile(true) => {
                    // acc was destroyed or newly created or if it is changed precompile, just add it to output.
                    out.insert(add, acc);
                }
                Filth::Dirty(_) => {
                    // check original and cleanup slots that are not dirty.
                    // In this case,return slots that are found in dirty_originals
                    let mut change = Map::new();
                    for &dirty_key in dirty.keys() {
                        change.insert(dirty_key, acc.storage.get(&dirty_key).cloned().unwrap());
                    }
                    acc.storage = change;
                    out.insert(add, acc);
                }
                _ => {}
            }
        }
        // state cleanup
        self.logs.clear();
        //println!(" changeset: {:?}", self.changelog);
        //assert!(self.changelog.len() == 1, "Changeset ");
        self.changelog = vec![Map::new(); 1];
        self.depth = 0;
        self.state = Map::new();
        out
    }

    /// Use it with load_account function.
    pub fn account(&self, address: H160) -> &Account {
        self.state.get(&address).unwrap() // Allways assume that acc is already loaded
    }

    pub fn depth(&self) -> u64 {
        self.depth as u64
    }

    /// use it only if you know that acc is hot
    pub fn set_code(&mut self, address: H160, code: Bytes, code_hash: H256) {
        let acc = self.log_dirty(address, |_| {});
        acc.info.code = Some(code);
        acc.info.code_hash = Some(code_hash);
    }

    pub fn inc_nonce(&mut self, address: H160) -> u64 {
        // asume account is hot and loaded
        let acc = self.log_dirty(address, |_| {});
        let old_nonce = acc.info.nonce;
        acc.info.nonce += 1;
        old_nonce
    }

    // log dirty change and return account back
    fn log_dirty<Fn: FnOnce(&mut DirtyChangeLog)>(
        &mut self,
        address: H160,
        update: Fn,
    ) -> &mut Account {
        let acc = self.state.get_mut(&address).unwrap();
        match self.changelog.last_mut().unwrap().entry(address) {
            Entry::Occupied(mut entry) => {
                if let ChangeLog::Dirty(changelog) = entry.get_mut() {
                    update(changelog);
                }
            }
            Entry::Vacant(entry) => {
                let changelog = if let Filth::Precompile(was_dirty) = acc.filth {
                    ChangeLog::PrecompileBalanceChange(acc.info.balance, was_dirty)
                } else {
                    let was_clean = matches!(acc.filth, Filth::Clean);
                    let mut changelog = DirtyChangeLog {
                        info: acc.info.clone(),
                        dirty_storage: Map::new(),
                        was_clean,
                    };
                    update(&mut changelog);
                    ChangeLog::Dirty(changelog)
                };

                entry.insert(changelog);
            }
        }
        acc.filth.make_dirty();
        acc
    }

    pub fn transfer<DB: Database>(
        &mut self,
        from: H160,
        to: H160,
        value: U256,
        db: &mut DB,
    ) -> Result<(bool, bool), ExitRevert> {
        // load accounts
        let from_is_cold = self.load_account(from, db);

        if value == U256::zero() {
            return Ok((from_is_cold, false));
        }

        let to_is_cold = self.load_account(to, db);
        // check from balance and substract value
        let from = self.log_dirty(from, |_| {});
        if from.info.balance < value {
            // TODO shold we remove it from log_dirty
            return Err(ExitRevert::OutOfFund);
        }
        from.info.balance -= value;

        let to = self.log_dirty(to, |_| {});
        to.info.balance += value;

        Ok((from_is_cold, to_is_cold))
    }

    pub fn create_checkpoint(&mut self) -> SubRoutineCheckpoint {
        let checkpoint = SubRoutineCheckpoint {
            log_i: self.logs.len(),
            changelog_i: self.changelog.len(),
            depth: self.depth,
        };
        self.depth += 1;
        self.changelog.push(Default::default());
        checkpoint
    }

    ///
    /// return if it has collition of addresses
    pub fn new_contract_acc<DB: Database>(
        &mut self,
        address: H160,
        precompiles: &[H160],
        db: &mut DB,
    ) -> bool {
        let (acc, _) = self.load_code(address, db);
        // check collision. Code exists
        if let Some(ref code) = acc.info.code {
            if !code.is_empty() {
                return false;
            }
        }
        // Check collision. Nonce is not zero
        if acc.info.nonce != 0 {
            return false;
        }

        // Check collision. New account address is precompile.
        if precompiles.contains(&address) {
            return false;
        }
        // TODO please check this. Does account that is destroyed is forbiden to be recreated again with create2?
        if matches!(acc.filth, Filth::Destroyed | Filth::NewlyCreated) {
            return false;
        }

        let original_balance = acc.info.balance;
        let original_filth = acc.filth.clone();
        acc.filth = Filth::NewlyCreated;
        // mark it in changelog as newly created
        self.changelog
            .last_mut()
            .unwrap()
            .entry(address)
            .or_insert_with(|| ChangeLog::Created(original_balance, original_filth));

        true
    }

    #[inline]
    fn revert_account_changelog(
        state: &mut State,
        add: H160,
        acc_change: ChangeLog,
    ) -> Option<Account> {
        match acc_change {
            // it was cold loaded. Remove it from global set
            ChangeLog::ColdLoaded => state.remove(&add),
            ChangeLog::PrecompileBalanceChange(original_balance, flag) => {
                let acc = state.get_mut(&add).unwrap();
                acc.info.balance = original_balance;
                acc.filth = Filth::Precompile(flag);
                None
            }
            ChangeLog::Destroyed(account) => {
                state.insert(add, account.clone()); // done
                Some(account)
            }
            ChangeLog::Created(balance, orig_filth) => {
                let acc = state.get_mut(&add).unwrap();
                acc.info.code = None;
                acc.info.code_hash = None;
                acc.info.nonce = 0;
                acc.info.balance = balance;
                acc.filth = orig_filth;
                acc.storage.clear();
                Some(acc.clone())
            }
            // if there are dirty changes in log
            ChangeLog::Dirty(dirty_log) => {
                let acc = state.get_mut(&add).unwrap(); // should be present

                // changset is clean,
                acc.info.balance = dirty_log.info.balance;
                acc.info.nonce = dirty_log.info.nonce;
                // BIG TODO filth
                if dirty_log.was_clean {
                    acc.filth = Filth::Clean;
                }
                if dirty_log.was_clean
                    || matches!(acc.filth, Filth::Destroyed | Filth::NewlyCreated)
                {
                    // Handle storage change
                    for (slot, log) in dirty_log.dirty_storage {
                        match log {
                            SlotChangeLog::Cold => {
                                acc.storage.remove(&slot);
                            }
                            SlotChangeLog::OriginalDirty(previous) => {
                                acc.storage.insert(slot, previous);
                            }
                        }
                    }
                } else {
                    let dirty = match &mut acc.filth {
                        Filth::Dirty(dirty) => dirty,
                        _ => panic!("panic this should not happen"),
                    };
                    for (slot, log) in dirty_log.dirty_storage {
                        match log {
                            SlotChangeLog::Cold => {
                                acc.storage.remove(&slot);
                                dirty.remove(&slot);
                            }
                            SlotChangeLog::OriginalDirty(previous) => {
                                // if it is marked as dirty that means we wrote something in this slot.
                                // and with that unwrap is okay.
                                let present = acc.storage.insert(slot, previous).unwrap();
                                match dirty.entry(slot) {
                                    Entry::Occupied(entry) => {
                                        if previous == *entry.get() {
                                            entry.remove();
                                        }
                                    }
                                    Entry::Vacant(entry) => {
                                        entry.insert(present);
                                    }
                                }
                            }
                        }
                    }
                }
                Some(acc.clone())
            }
        }
    }

    fn revert_changelog(state: &mut State, changelog: Map<H160, ChangeLog>) {
        for (add, acc_change) in changelog {
            Self::revert_account_changelog(state, add, acc_change);
        }
    }

    pub fn depth_decs(&mut self) {
        self.depth -= 1;
    }

    pub fn checkpoint_revert(&mut self, checkpoint: SubRoutineCheckpoint) {
        let state = &mut self.state;
        // iterate over last N changelogs and revert it to our global state
        let leng = self.changelog.len();
        self.changelog
            .iter_mut()
            .rev()
            .take(leng - checkpoint.changelog_i)
            .for_each(|cs| Self::revert_changelog(state, mem::take(cs)));

        self.logs.truncate(checkpoint.log_i);
        self.changelog.truncate(checkpoint.changelog_i);
    }

    /// transfer balance from address to target. Check if target exist/is_cold
    pub fn selfdestruct<DB: Database>(
        &mut self,
        address: H160,
        target: H160,
        db: &mut DB,
    ) -> SelfDestructResult {
        let (is_cold, exists) = self.load_account_exist(target, db);
        // transfer all the balance
        let (had_value, previously_destroyed) = {
            let acc = self.state.get_mut(&address).unwrap();
            let value = acc.info.balance;
            let had_value = value != U256::zero();
            let previously_destroyed = matches!(acc.filth, Filth::Destroyed);
            let target = self.log_dirty(target, |_| {});
            target.info.balance += value;
            (had_value, previously_destroyed)
        };
        // if there is no account in this subroutine changelog,
        // save it so that it can be restored if subroutine needs to be rediscarded.
        let acc = match self.changelog.last_mut().unwrap().entry(address) {
            Entry::Occupied(mut entry) => {
                let changelog = entry.get_mut();
                // revert current changelog
                let acc =
                    Self::revert_account_changelog(&mut self.state, address, changelog.clone())
                        .unwrap();
                // mark it as destryoted and save original account if needed to revert it.
                *changelog = ChangeLog::Destroyed(acc.clone());
                self.state.entry(address).or_insert(acc)
            }
            Entry::Vacant(entry) => {
                // no change logs. Take current state acc and save it
                let acc = self.state.get_mut(&address).unwrap();
                entry.insert(ChangeLog::Destroyed(acc.clone()));
                acc
            }
        };
        // nulify state and info. Mark acc as destroyed.
        acc.filth = Filth::Destroyed;
        acc.storage = Map::new();
        acc.info = AccountInfo::default();
        SelfDestructResult {
            had_value,
            is_cold,
            exists,
            previously_destroyed,
        }
    }

    /// load account into memory. return if it is cold or hot accessed
    pub fn load_account<'a, DB: Database>(&'a mut self, address: H160, db: &mut DB) -> bool {
        let is_cold = match self.state.entry(address.clone()) {
            Entry::Occupied::<'a>(ref mut entry) => false,
            Entry::Vacant::<'a>(vac) => {
                let acc: Account = db.basic(address).into();
                // insert none in changelog that represent that we just loaded this acc in this subroutine
                self.changelog
                    .last_mut()
                    .unwrap()
                    .insert(address.clone(), ChangeLog::ColdLoaded);
                vac.insert(acc.clone());
                true
            }
        };
        is_cold
    }

    // first is is_cold second bool is exists.
    pub fn load_account_exist<DB: Database>(&mut self, address: H160, db: &mut DB) -> (bool, bool) {
        let (acc, is_cold) = self.load_code(address, db);
        let info = acc.info.clone();
        let is_empty = info.balance == U256::zero()
            && info.nonce == 0
            && info.code.unwrap_or_default() == Bytes::default();
        (is_cold, !is_empty)
    }

    pub fn load_code<DB: Database>(&mut self, address: H160, db: &mut DB) -> (&mut Account, bool) {
        let is_cold = self.load_account(address.clone(), db);
        let acc = self.state.get_mut(&address).unwrap();
        let dont_load_from_db = !matches!(acc.filth, Filth::Destroyed | Filth::NewlyCreated);
        if dont_load_from_db && acc.info.code.is_none() {
            let code = if let Some(code_hash) = acc.info.code_hash {
                db.code_by_hash(code_hash)
            } else {
                db.code(address)
            };
            acc.info.code = Some(code);
        }
        (acc, is_cold)
    }

    // account is already present and loaded.
    pub fn sload<DB: Database>(&mut self, address: H160, index: H256, db: &mut DB) -> (H256, bool) {
        let acc = self.state.get_mut(&address).unwrap(); // asume acc is hot
        let load = match acc.storage.entry(index) {
            Entry::Occupied(occ) => (occ.get().clone(), false),
            // add slot to ColdLoaded in changelog
            Entry::Vacant(vac) => {
                // if storage was destroyed, we dont need to ping db.
                let value = if matches!(acc.filth, Filth::Destroyed | Filth::NewlyCreated) {
                    H256::zero()
                } else {
                    db.storage(address, index)
                };
                // add it to changelog as cold loaded.
                match self.changelog.last_mut().unwrap().entry(address) {
                    Entry::Occupied(mut entry) => {
                        // this is usual route to take.
                        if let ChangeLog::Dirty(dirty) = entry.get_mut() {
                            dirty
                                .dirty_storage
                                .entry(index)
                                .or_insert(SlotChangeLog::Cold);
                        }
                    }
                    Entry::Vacant(entry) => {
                        // if account is not found in log. Insert log account. and add cold access to slot.
                        // this can happen if we previously loaded acc and now want to access it again.
                        let mut dirty = DirtyChangeLog {
                            info: acc.info.clone(),
                            dirty_storage: Map::new(),
                            was_clean: matches!(acc.filth, Filth::Clean),
                        };
                        dirty.dirty_storage.insert(index, SlotChangeLog::Cold);
                        entry.insert(ChangeLog::Dirty(dirty));
                    }
                }
                vac.insert(value);

                (value, true)
            }
        };
        load
    }

    /// account should already be present in our state.
    /// returns (original,present,new) slot
    pub fn sstore<DB: Database>(
        &mut self,
        address: H160,
        index: H256,
        new: H256,
        db: &mut DB,
    ) -> (H256, H256, H256, bool) {
        // assume that acc exists and load the slot.
        let (present, is_cold) = self.sload(address, index, db);
        //println!("sstore:{:?}:{:?}({:?})=>{:?}::{:?}",address,index,present,new,is_cold);
        let acc = self.state.get_mut(&address).unwrap();
        // if there is no original value in dirty return present valuem that is our original.
        let original = if let Some(original) = acc.filth.original_slot(index) {
            original
        } else {
            present
        };
        // new value is same as present, we dont need to do anything
        if present == new {
            return (original, present, new, is_cold);
        }

        // if clean tag it as dirty. If original value not found, insert present value (it is original).
        acc.filth.insert_dirty_original(index, present);
        // insert value into present state.
        acc.storage.insert(index, new);

        // insert present value inside changelog so that it can be reverted if needed.
        self.log_dirty(address, |dirty_log| {
            if let Entry::Vacant(entry) = dirty_log.dirty_storage.entry(index) {
                entry.insert(SlotChangeLog::OriginalDirty(present));
            }
        });

        (original, present, new, is_cold)
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
    pub storage: Map<H256, H256>,
    /// is account info is dirty, destroyed or clean.
    /// if selfdestruct opcode is called, destroyed flag will be true. If true we dont need to fetch slot from DB.
    /// dirty flag contains list of original value and this is used to determent if slot was changed
    /// and for calcuation of gas cunsumption.
    pub filth: Filth,
}

impl Account {
    pub fn is_empty(&self) -> bool {
        let code_empty = if let Some(ref code) = self.info.code {
            code.is_empty()
        } else {
            true
        };
        self.info.balance == U256::zero() && self.info.nonce == 0 && code_empty
    }
}

impl From<AccountInfo> for Account {
    fn from(info: AccountInfo) -> Self {
        Self {
            info,
            storage: Map::new(),
            filth: Filth::Clean,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Filth {
    /// clean load from db
    Clean,
    ///  original state, and contains slots with original values that we changed.
    Dirty(Map<H256, H256>),
    /// destroyed by selfdestruct or it is newly
    ///  created by create/create2 opcode. Either way dont save original values
    Destroyed,

    NewlyCreated,
    // precompile
    // bool signals if it is dirty or not
    Precompile(bool),
}

impl Filth {
    pub fn abandon_old_storage(&self) -> bool {
        matches!(self, Self::Destroyed | Self::NewlyCreated)
    }
    /// insert into dirty flag and return if slot was already dirty or not.
    #[inline]
    pub fn insert_dirty_original(&mut self, index: H256, present_value: H256) {
        match self {
            Self::Clean => {
                let mut map = Map::new();
                map.insert(index, present_value);
                *self = Self::Dirty(map);
            }
            Self::Dirty(ref mut originals) => {
                // insert only if not present. If present it is assumed with always have original value.
                originals.entry(index).or_insert(present_value);
            }
            Self::Destroyed | Self::NewlyCreated => (),
            Self::Precompile(_) => panic!("Insert dirty for precompile is not possible"),
        }
    }
    pub fn original_slot(&mut self, index: H256) -> Option<H256> {
        match self {
            Self::Clean | Self::Precompile(_) => None,
            Self::Dirty(ref originals) => originals.get(&index).cloned(),
            Self::Destroyed | Self::NewlyCreated => Some(H256::zero()),
        }
    }

    pub fn clean(&mut self) -> Map<H256, H256> {
        match self {
            Self::Dirty(out) => mem::replace(out, Map::new()),
            _ => Map::new(),
        }
    }

    pub fn make_dirty(&mut self) {
        match self {
            Self::Clean => *self = Self::Dirty(Map::new()),
            Self::Precompile(flag) => *flag = true,
            _ => (),
        }
    }
}
