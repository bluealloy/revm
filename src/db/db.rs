use crate::collection::{vec::Vec, Map};

use primitive_types::{H160, H256, U256};

use super::trie;
use crate::{Account, AccountInfo, Log};
use bytes::Bytes;

pub trait Database {
    /// Whether account at address exists.
    fn exists(&mut self, address: H160) -> Option<AccountInfo>;
    /// Get basic account information.
    fn basic(&mut self, address: H160) -> AccountInfo;
    /// Get account code.
    fn code(&mut self, address: H160) -> Bytes;
    /// Get account code by its hash
    fn code_by_hash(&mut self, code_hash: H256) -> Bytes;
    /// Get storage value of address at index.
    fn storage(&mut self, address: H160, index: H256) -> H256;

    // History related
    fn block_hash(&mut self, number: U256) -> H256;

    //apply
    //traces
}

/// Memory backend, storing all state values in a `Map` in memory.
#[derive(Debug, Clone)]
pub struct StateDB {
    cache: Map<H160, AccountInfo>,
    storage: Map<H160, Map<H256, H256>>,
    logs: Vec<Log>,
}

impl StateDB {
    pub fn insert_cache(&mut self, address: H160, account: AccountInfo) {
        self.cache.insert(address, account);
    }

    pub fn insert_cache_storage(&mut self, address: H160, slot: H256, value: H256) {
        self.storage.entry(address).or_default().insert(slot, value);
    }

    pub fn apply(&mut self, changes: Map<H160, Account>) {
        for (add, acc) in changes {
            if acc.is_empty() {
                self.cache.remove(&add);
            } else {
                self.cache.insert(add, acc.info);
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

    pub fn state_root(&self) -> H256 {
        let vec = self
            .cache
            .iter()
            .map(|(address, info)| {
                let storage = self.storage.get(address).cloned().unwrap_or_default();
                let storage_root = trie::trie_account_rlp(info, storage);
                (address.clone(), storage_root)
            })
            .collect();

        trie::trie_root(vec)
    }

    /// Create a new memory backend.
    pub fn new() -> Self {
        Self {
            cache: Map::new(),
            storage: Map::new(),
            logs: Vec::new(),
        }
    }

    /// return true if account exists or fetch it from database
    fn fetch_account(&self, address: &H160) -> bool {
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

impl Database for StateDB {
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
        //log::info!(target: "evm::handler", "{:?} exists",address);
        if self.fetch_account(&address) {
            Some(self.cache.get(&address).cloned().unwrap())
        } else {
            None
        }
    }

    fn basic(&mut self, address: H160) -> AccountInfo {
        //log::info!(target: "evm::handler", "{:?} basic acc info",address);
        if self.fetch_account(&address) {
            self.cache.get(&address).cloned().unwrap()
        } else {
            AccountInfo::default()
        }
    }

    fn code(&mut self, address: H160) -> Bytes {
        //log::info!(target: "evm::handler", "{:?} code",address);
        if self.fetch_account(&address) {
            let acc = self.cache.get_mut(&address).unwrap();
            if let Some(ref code) = acc.code {
                return code.clone();
            }
            if acc.code_hash.is_none() {
                return Bytes::new();
            }
            Bytes::new()
            /*let code = self.db.contract(&acc.code_hash.unwrap());
            if code.is_none() {
                return Bytes::new();
            }
            let code = code.unwrap();
            acc.code = Some(code.clone());
            code*/
        } else {
            Bytes::new()
        }
    }

    fn storage(&mut self, address: H160, index: H256) -> H256 {
        //log::info!(target: "evm::handler", "{:?} storage index {:?}",address, index);
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

    fn code_by_hash(&mut self, _code_hash: H256) -> Bytes {
        todo!()
    }
}
