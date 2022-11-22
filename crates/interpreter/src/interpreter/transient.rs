use crate::primitives::{B160, U256};
use std::collections::HashMap;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TransientStorage {
    data: HashMap<B160, HashMap<U256, U256>>,
}

impl Default for TransientStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl TransientStorage {
    pub fn new() -> Self {
        Self {
            data: HashMap::default(),
        }
    }

    pub fn set(&mut self, address: B160, key: U256, value: U256) {
        match self.data.get_mut(&address) {
            Some(storage) => {
                let _ = storage.insert(key, value);
                return;
            }
            None => {
                let mut storage: HashMap<U256, U256> = HashMap::default();
                let _ = storage.insert(key, value);
                self.data.insert(address, storage);
                return;
            }
        }
    }

    pub fn get(&self, address: B160, key: U256) -> U256 {
        match self.data.get(&address) {
            Some(storage) => match storage.get(&key) {
                Some(value) => *value,
                None => U256::default(),
            },
            None => U256::default(),
        }
    }
}
