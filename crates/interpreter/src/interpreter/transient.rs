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
        self.data
            .get_mut(&address)
            .and_then(|s| s.insert(key, value));
    }

    pub fn get(&self, address: B160, key: U256) -> U256 {
        self.data
            .get(&address)
            .and_then(|s| s.get(&key))
            .cloned()
            .unwrap_or_default()
    }
}
