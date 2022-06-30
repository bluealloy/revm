use crate::{Database, Filth, KECCAK_EMPTY};

use alloc::{
    collections::btree_map::{self, BTreeMap},
    vec::Vec,
};
use hashbrown::{hash_map::Entry, HashMap as Map};

use primitive_types::{H160, H256, U256};

use crate::{Account, AccountInfo, Log};
use bytes::Bytes;
use sha3::{Digest, Keccak256};

use super::{DatabaseCommit, DatabaseRef};

pub type InMemoryDB = CacheDB<EmptyDB>;

impl InMemoryDB {
    pub fn default() -> Self {
        CacheDB::new(EmptyDB {})
    }
}

/// Memory backend, storing all state values in a `Map` in memory.
#[derive(Debug, Clone)]
pub struct CacheDB<ExtDB: DatabaseRef> {
    /// Dummy account info where `code` is always `None`.
    /// Code bytes can be found in `contracts`.
    accounts: BTreeMap<H160, DbAccount>,
    contracts: Map<H256, Bytes>,
    logs: Vec<Log>,
    block_hashes: Map<U256, H256>,
    db: ExtDB,
}

#[derive(Debug, Clone)]
pub struct DbAccount {
    pub info: AccountInfo,
    /// If account is selfdestructed or newly created, storage will be cleared.
    pub account_state: AccountState,
    /// storage slots
    pub storage: BTreeMap<U256, U256>,
}

#[derive(Debug, Clone)]
pub enum AccountState {
    /// EVM touched this account
    EVMTouched,
    /// EVM cleared storage of this account, mostly by selfdestruct
    EVMStorageCleared,
    /// EVM didnt interacted with this account
    None,
}

impl<ExtDB: DatabaseRef> CacheDB<ExtDB> {
    pub fn new(db: ExtDB) -> Self {
        let mut contracts = Map::new();
        contracts.insert(KECCAK_EMPTY, Bytes::new());
        contracts.insert(H256::zero(), Bytes::new());
        Self {
            accounts: BTreeMap::new(),
            contracts,
            logs: Vec::default(),
            block_hashes: Map::new(),
            db,
        }
    }
    pub fn accounts(&self) -> &BTreeMap<H160, DbAccount> {
        &self.accounts
    }

    fn insert_contract(&mut self, account: &mut AccountInfo) {
        let code = core::mem::take(&mut account.code);
        if let Some(code) = code {
            if !code.is_empty() {
                let code_hash = H256::from_slice(&Keccak256::digest(&code));
                account.code_hash = code_hash;
                self.contracts.insert(code_hash, code);
            }
        }
        if account.code_hash.is_zero() {
            account.code_hash = KECCAK_EMPTY;
        }
    }

    pub fn insert_cache(&mut self, address: H160, mut info: AccountInfo) {
        self.insert_contract(&mut info);
        self.accounts.insert(
            address,
            DbAccount {
                info,
                account_state: AccountState::None,
                storage: BTreeMap::new(),
            },
        );
    }

    pub fn insert_cache_storage(&mut self, address: H160, slot: U256, value: U256) {
        self.accounts
            .entry(address)
            .or_insert(DbAccount {
                info: AccountInfo::default(),
                account_state: AccountState::None,
                storage: BTreeMap::new(),
            })
            .storage
            .insert(slot, value);
    }

    pub fn db(&self) -> &ExtDB {
        &self.db
    }

    pub fn db_mut(&mut self) -> &mut ExtDB {
        &mut self.db
    }
}

impl<ExtDB: DatabaseRef> DatabaseCommit for CacheDB<ExtDB> {
    fn commit(&mut self, changes: Map<H160, Account>) {
        for (add, acc) in changes {
            if acc.is_empty() || matches!(acc.filth, Filth::Destroyed) {
                // clear account data, and increate its incarnation.
                let acc = self.accounts.entry(add).or_insert(DbAccount {
                    info: AccountInfo::default(),
                    storage: BTreeMap::new(),
                    account_state: AccountState::EVMStorageCleared,
                });
                acc.account_state = AccountState::EVMStorageCleared;
                acc.storage = BTreeMap::new();
                acc.info = AccountInfo::default();
            } else {
                match self.accounts.entry(add) {
                    btree_map::Entry::Vacant(entry) => {
                        // can happend if new account is created
                        entry.insert(DbAccount {
                            info: acc.info,
                            account_state: AccountState::EVMTouched,
                            storage: acc.storage.into_iter().collect(),
                        });
                    }
                    btree_map::Entry::Occupied(mut entry) => {
                        let db_acc = entry.get_mut();
                        db_acc.info = acc.info;
                        if matches!(acc.filth, Filth::NewlyCreated) {
                            db_acc.account_state = AccountState::EVMStorageCleared;
                            db_acc.storage = acc.storage.into_iter().collect();
                        } else {
                            if matches!(db_acc.account_state, AccountState::None) {
                                db_acc.account_state = AccountState::EVMTouched;
                            }
                            db_acc.storage.extend(acc.storage);
                        }
                    }
                }
            }
        }
    }
}

impl<ExtDB: DatabaseRef> Database for CacheDB<ExtDB> {
    fn block_hash(&mut self, number: U256) -> H256 {
        match self.block_hashes.entry(number) {
            Entry::Occupied(entry) => *entry.get(),
            Entry::Vacant(entry) => {
                let hash = self.db.block_hash(number);
                entry.insert(hash);
                hash
            }
        }
    }

    fn basic(&mut self, address: H160) -> AccountInfo {
        match self.accounts.entry(address) {
            btree_map::Entry::Occupied(entry) => entry.get().info.clone(),
            btree_map::Entry::Vacant(entry) => {
                let info = self.db.basic(address);
                entry.insert(DbAccount {
                    info: info.clone(),
                    account_state: AccountState::EVMTouched,
                    storage: BTreeMap::new(),
                });
                info
            }
        }
    }

    /// Get the value in an account's storage slot.
    ///
    /// It is assumed that account is already loaded.
    fn storage(&mut self, address: H160, index: U256) -> U256 {
        match self.accounts.entry(address) {
            btree_map::Entry::Occupied(mut acc_entry) => {
                let acc_entry = acc_entry.get_mut();
                match acc_entry.storage.entry(index) {
                    btree_map::Entry::Occupied(entry) => *entry.get(),
                    btree_map::Entry::Vacant(entry) => {
                        if matches!(acc_entry.account_state, AccountState::EVMStorageCleared) {
                            U256::zero()
                        } else {
                            let slot = self.db.storage(address, index);
                            entry.insert(slot);
                            slot
                        }
                    }
                }
            }
            btree_map::Entry::Vacant(acc_entry) => {
                // acc needs to be loaded for us to access slots.
                let info = self.db.basic(address);
                let value = self.db.storage(address, index);
                acc_entry.insert(DbAccount {
                    info,
                    account_state: AccountState::EVMTouched,
                    storage: BTreeMap::from([(index, value)]),
                });
                value
            }
        }
    }

    fn code_by_hash(&mut self, code_hash: H256) -> Bytes {
        match self.contracts.entry(code_hash) {
            Entry::Occupied(entry) => entry.get().clone(),
            Entry::Vacant(entry) => {
                // if you return code bytes when basic fn is called this function is not needed.
                entry.insert(self.db.code_by_hash(code_hash)).clone()
            }
        }
    }
}

impl<ExtDB: DatabaseRef> DatabaseRef for CacheDB<ExtDB> {
    fn block_hash(&self, number: U256) -> H256 {
        match self.block_hashes.get(&number) {
            Some(entry) => *entry,
            None => self.db.block_hash(number),
        }
    }

    fn basic(&self, address: H160) -> AccountInfo {
        match self.accounts.get(&address) {
            Some(acc) => acc.info.clone(),
            None => self.db.basic(address),
        }
    }

    fn storage(&self, address: H160, index: U256) -> U256 {
        match self.accounts.get(&address) {
            Some(acc_entry) => match acc_entry.storage.get(&index) {
                Some(entry) => *entry,
                None => {
                    if matches!(acc_entry.account_state, AccountState::EVMStorageCleared) {
                        U256::zero()
                    } else {
                        self.db.storage(address, index)
                    }
                }
            },
            None => self.db.storage(address, index),
        }
    }

    fn code_by_hash(&self, code_hash: H256) -> Bytes {
        match self.contracts.get(&code_hash) {
            Some(entry) => entry.clone(),
            None => self.db.code_by_hash(code_hash),
        }
    }
}

/// An empty database that always returns default values when queried.
#[derive(Debug, Default, Clone)]
pub struct EmptyDB();

impl DatabaseRef for EmptyDB {
    /// Get basic account information.
    fn basic(&self, _address: H160) -> AccountInfo {
        AccountInfo::default()
    }
    /// Get account code by its hash
    fn code_by_hash(&self, _code_hash: H256) -> Bytes {
        Bytes::default()
    }
    /// Get storage value of address at index.
    fn storage(&self, _address: H160, _index: U256) -> U256 {
        U256::default()
    }

    // History related
    fn block_hash(&self, number: U256) -> H256 {
        let mut buffer: [u8; 4 * 8] = [0; 4 * 8];
        number.to_big_endian(&mut buffer);
        H256::from_slice(&Keccak256::digest(&buffer))
    }
}

/// Custom benchmarking DB that only has account info for the zero address.
///
/// Any other address will return an empty account.
#[derive(Debug, Default, Clone)]
pub struct BenchmarkDB(pub Bytes);

impl Database for BenchmarkDB {
    /// Get basic account information.
    fn basic(&mut self, address: H160) -> AccountInfo {
        if address == H160::zero() {
            return AccountInfo {
                nonce: 1,
                balance: U256::from(10000000),
                code: Some(self.0.clone()),
                code_hash: KECCAK_EMPTY,
            };
        }
        AccountInfo::default()
    }

    /// Get account code by its hash
    fn code_by_hash(&mut self, _code_hash: H256) -> Bytes {
        Bytes::default()
    }

    /// Get storage value of address at index.
    fn storage(&mut self, _address: H160, _index: U256) -> U256 {
        U256::default()
    }

    // History related
    fn block_hash(&mut self, _number: U256) -> H256 {
        H256::default()
    }
}
