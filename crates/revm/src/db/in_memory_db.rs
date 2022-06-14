use crate::{Database, Filth, KECCAK_EMPTY};

use alloc::vec::Vec;
use hashbrown::{hash_map::Entry, HashMap as Map, HashMap};

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
    changes: Map<H160, AccountInfo>,
    cache: Map<H160, AccountInfo>,
    storage: Map<H160, Map<U256, U256>>,
    contracts: Map<H256, Bytes>,
    logs: Vec<Log>,
    block_hashes: Map<U256, H256>,
    db: ExtDB,
}

impl<ExtDB: DatabaseRef> CacheDB<ExtDB> {
    pub fn new(db: ExtDB) -> Self {
        let mut contracts = Map::new();
        contracts.insert(KECCAK_EMPTY, Bytes::new());
        contracts.insert(H256::zero(), Bytes::new());
        Self {
            changes: Map::new(),
            cache: Map::new(),
            storage: Map::new(),
            contracts,
            logs: Vec::default(),
            block_hashes: Map::new(),
            db,
        }
    }
    pub fn changes(&self) -> &Map<H160, AccountInfo> {
        &self.changes
    }

    pub fn cache(&self) -> &Map<H160, AccountInfo> {
        &self.cache
    }

    pub fn storage(&self) -> &Map<H160, Map<U256, U256>> {
        &self.storage
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

    pub fn insert_change(&mut self, address: H160, mut account: AccountInfo) {
        self.insert_contract(&mut account);
        self.changes.insert(address, account);
    }

    pub fn insert_cache(&mut self, address: H160, mut account: AccountInfo) {
        self.insert_contract(&mut account);
        self.cache.insert(address, account);
    }

    pub fn insert_cache_storage(&mut self, address: H160, slot: U256, value: U256) {
        self.storage.entry(address).or_default().insert(slot, value);
    }

    pub fn db(&self) -> &ExtDB {
        &self.db
    }

    pub fn db_mut(&mut self) -> &mut ExtDB {
        &mut self.db
    }
}

// TODO It is currently only committing to cached in-memory DB
impl<ExtDB: DatabaseRef> DatabaseCommit for CacheDB<ExtDB> {
    fn commit(&mut self, changes: Map<H160, Account>) {
        // clear storage by setting all values to `0`
        fn clear_storage(storage: &mut HashMap<U256, U256>) {
            storage.values_mut().for_each(|val| {
                *val = U256::zero();
            })
        }

        for (add, acc) in changes {
            if acc.is_empty() || matches!(acc.filth, Filth::Destroyed) {
                // clear account data, but don't remove entry
                self.changes.insert(add, Default::default());
                self.storage.get_mut(&add).map(clear_storage);
            } else {
                self.insert_change(add, acc.info);
                let storage = self.storage.entry(add).or_default();
                if acc.filth.abandon_old_storage() {
                    clear_storage(storage);
                }
                storage.extend(acc.storage);
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
        if let Some(changed) = self.changes.get(&address) {
            changed.clone()
        } else {
            match self.cache.entry(address) {
                Entry::Occupied(entry) => entry.get().clone(),
                Entry::Vacant(entry) => {
                    let acc = self.db.basic(address);
                    if !acc.is_empty() {
                        entry.insert(acc.clone());
                    }
                    acc
                }
            }
        }
    }

    /// Get the value in an account's storage slot.
    ///
    /// It is assumed that account is already loaded.
    fn storage(&mut self, address: H160, index: U256) -> U256 {
        match self.storage.entry(address) {
            Entry::Occupied(mut entry) => match entry.get_mut().entry(index) {
                Entry::Occupied(entry) => *entry.get(),
                Entry::Vacant(entry) => {
                    let slot = self.db.storage(address, index);
                    entry.insert(slot);
                    slot
                }
            },
            Entry::Vacant(entry) => {
                let mut storage = Map::new();
                let slot = self.db.storage(address, index);
                storage.insert(index, slot);
                entry.insert(storage);
                slot
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
        match self.cache.get(&address) {
            Some(info) => info.clone(),
            None => self.db.basic(address),
        }
    }

    fn storage(&self, address: H160, index: U256) -> U256 {
        match self.storage.get(&address) {
            Some(entry) => match entry.get(&index) {
                Some(entry) => *entry,
                None => self.db.storage(address, index),
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
