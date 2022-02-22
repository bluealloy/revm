use crate::{subroutine::Filth, Database, KECCAK_EMPTY};

use alloc::vec::Vec;
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
    cache: Map<H160, AccountInfo>,
    storage: Map<H160, Map<U256, U256>>,
    contracts: Map<H256, Bytes>,
    logs: Vec<Log>,
    db: ExtDB,
}

impl<ExtDB: DatabaseRef> CacheDB<ExtDB> {
    pub fn new(db: ExtDB) -> Self {
        let mut contracts = Map::new();
        contracts.insert(KECCAK_EMPTY, Bytes::new());
        contracts.insert(H256::zero(), Bytes::new());
        Self {
            cache: Map::new(),
            storage: Map::new(),
            contracts,
            logs: Vec::default(),
            db,
        }
    }

    pub fn cache(&self) -> &Map<H160, AccountInfo> {
        &self.cache
    }

    pub fn storage(&self) -> &Map<H160, Map<U256, U256>> {
        &self.storage
    }

    pub fn insert_cache(&mut self, address: H160, mut account: AccountInfo) {
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
        self.cache.insert(address, account);
    }

    pub fn insert_cache_storage(&mut self, address: H160, slot: U256, value: U256) {
        self.storage.entry(address).or_default().insert(slot, value);
    }
}

// TODO It is currently only commiting to cached in-memory DB
impl<ExtDB: DatabaseRef> DatabaseCommit for CacheDB<ExtDB> {
    fn commit(&mut self, changes: Map<H160, Account>) {
        for (add, acc) in changes {
            if acc.is_empty() || matches!(acc.filth, Filth::Destroyed) {
                self.cache.remove(&add);
                self.storage.remove(&add);
            } else {
                self.insert_cache(add, acc.info);
                let storage = self.storage.entry(add).or_default();
                if acc.filth.abandon_old_storage() {
                    storage.clear();
                }
                for (index, value) in acc.storage {
                    if value.is_zero() {
                        storage.remove(&index);
                    } else {
                        storage.insert(index, value);
                    }
                }
                if storage.is_empty() {
                    self.storage.remove(&add);
                }
            }
        }
    }
}

impl<ExtDB: DatabaseRef> Database for CacheDB<ExtDB> {
    fn block_hash(&mut self, number: U256) -> H256 {
        self.db.block_hash(number)
    }

    fn basic(&mut self, address: H160) -> AccountInfo {
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
    fn block_hash(&self, _number: U256) -> H256 {
        H256::default()
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
