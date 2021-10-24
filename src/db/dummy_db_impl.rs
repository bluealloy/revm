use crate::{
    collection::{vec::Vec, Entry, Map},
    subroutine::Filth,
    Database, KECCAK_EMPTY,
};

use primitive_types::{H160, H256, U256};

use sha3::{Digest, Keccak256};
use crate::{Account, AccountInfo, Log};
use bytes::Bytes;

/// Memory backend, storing all state values in a `Map` in memory.
#[derive(Debug, Clone)]
pub struct DummyStateDB {
    /// dummy account info where code is allways None. Code bytes can be found in `contracts`
    cache: Map<H160, AccountInfo>,
    storage: Map<H160, Map<H256, H256>>,
    contracts: Map<H256, Bytes>,
    logs: Vec<Log>,
}

impl DummyStateDB {
    pub fn cache(&self) -> &Map<H160,AccountInfo> {
        &self.cache
    }
    pub fn storage(&self) -> &Map<H160,Map<H256,H256>> {
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
        // TODO see to remove zero from revm so that we dont need to do this.
        // it fails with selfdestruct
        if account.code_hash == H256::zero() {
            account.code_hash = KECCAK_EMPTY;
        }
        self.cache.insert(address, account);
    }

    pub fn insert_cache_storage(&mut self, address: H160, slot: H256, value: H256) {
        self.storage.entry(address).or_default().insert(slot, value);
    }

    pub fn apply(&mut self, changes: Map<H160, Account>) {
        for (add, acc) in changes {
            if acc.is_empty() || matches!(acc.filth, Filth::Destroyed) {
                self.cache.remove(&add);
                self.storage.remove(&add);
            } else {
                self.insert_cache(add, acc.info);
                let storage = self.storage.entry(add.clone()).or_default();
                if acc.filth.abandon_old_storage() {
                    storage.clear();
                }
                for (index, value) in acc.storage {
                    if value == H256::zero() {
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

    /// Create a new memory backend.
    pub fn new() -> Self {
        let mut contracts = Map::new();
        contracts.insert(KECCAK_EMPTY, Bytes::new());
        contracts.insert(H256::zero(), Bytes::new());
        Self {
            cache: Map::new(),
            storage: Map::new(),
            contracts,
            logs: Vec::new(),
        }
    }

    /// return true if account exists or fetch it from database
    fn fetch_account(&mut self, address: &H160) -> bool {
        {
            if let Some(acc) = self.cache.get(address) {
                return acc.exists();
            }
        }
        false

        // let (acc, exists) = if let Some(acc) = self.db.account(&ethH160::from_slice(&address.0)) {
        //     println!("FETCHING ACC");
        //     (CachedAccount::from(acc), true)
        // } else {
        //     (CachedAccount::default(), false)
        // };
        // self.cache.insert(address.clone(), acc);
        // exists
    }
}

impl Database for DummyStateDB {
    fn block_hash(&mut self, _number: U256) -> H256 {
        // if number >= self.vicinity.block_number
        // 	|| self.vicinity.block_number - number - U256::one()
        // 		>= U256::from(self.vicinity.block_hashes.len())
        // {
        // 	H256::default()
        // } else {
        // 	let index = (self.vicinity.block_number - number - U256::one()).as_usize();
        // 	self.vicinity.block_hashes[index]
        // }
        // TODO change to tx hash
        H256::zero()
    }

    fn exists(&mut self, address: H160) -> Option<AccountInfo> {
        if self.fetch_account(&address) {
            Some(self.cache.get(&address).cloned().unwrap())
        } else {
            None
        }
    }

    fn basic(&mut self, address: H160) -> AccountInfo {
        if self.fetch_account(&address) {
            let mut basic = self.cache.get(&address).cloned().unwrap();
            basic.code = None;
            basic
        } else {
            AccountInfo::default()
        }
    }

    fn storage(&mut self, address: H160, index: H256) -> H256 {
        if self.fetch_account(&address) {
            if let Some(storage) = self.storage.get(&address) {
                if let Some(slot) = storage.get(&index) {
                    return slot.clone();
                }
            }
            H256::zero()
            /*
            if let Some((_, storage)) = acc..get(&index) {
                return *storage;
            }
            let eth_address = H160::from(address.0);
            let eth_index = H256::from(index.0);
            let storage = self
                .db
                .storage(&eth_address, acc.incarnation, &eth_index)
                .map(|storage| H256::from(storage.0))
                .unwrap_or_default();
            acc.storage.insert(index, (false, storage));
            storage*/
        } else {
            H256::zero()
        }
    }

    fn code_by_hash(&mut self, code_hash: H256) -> Bytes {
        match self.contracts.entry(code_hash) {
            Entry::Occupied(entry) => entry.get().clone(),
            Entry::Vacant(_entry) => {
                // TODO fetch from db
                Bytes::new()
            }
        }
    }
}
