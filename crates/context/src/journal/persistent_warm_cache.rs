use primitives::{Address, HashMap, HashSet, StorageKey};

/// Tracks addresses and storage slots that have been accessed.
/// Persists across transactions for the lifetime of the EVM instance.
///
/// The cache is never cleared - it lives for the lifetime of the EVM instance,
/// which in practice is one block (e.g., in Reth, a fresh EVM is created per block).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PersistentWarmCache {
    warm_storage: HashMap<Address, HashSet<StorageKey>>,
}

impl PersistentWarmCache {
    /// Creates a new empty persistent warm cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Marks an account and its storage keys as warm.
    ///
    /// If the account is already warm, the storage keys are added to the existing set.
    /// Pass an empty iterator to warm only the account without any storage slots.
    pub fn warm_account_and_storage(
        &mut self,
        address: Address,
        storage_keys: impl IntoIterator<Item = StorageKey>,
    ) {
        let set = self.warm_storage.entry(address).or_default();
        for key in storage_keys {
            set.insert(key);
        }
    }

    /// Check if an address is warm.
    pub fn is_address_warm(&self, address: &Address) -> bool {
        self.warm_storage.contains_key(address)
    }

    /// Check if a storage slot is warm for the given address.
    pub fn is_storage_warm(&self, address: &Address, key: &StorageKey) -> bool {
        self.warm_storage
            .get(address)
            .map(|keys| keys.contains(key))
            .unwrap_or(false)
    }

    /// Mark an address as warm without any storage slots. (test only)
    #[cfg(test)]
    pub fn warm_address(&mut self, address: Address) {
        self.warm_storage.entry(address).or_default();
    }

    /// Mark a storage slot as warm. (test only)
    #[cfg(test)]
    pub fn warm_storage(&mut self, address: Address, key: StorageKey) {
        self.warm_storage.entry(address).or_default().insert(key);
    }

    /// Count of warm addresses. (test only)
    #[cfg(test)]
    pub fn warm_address_count(&self) -> usize {
        self.warm_storage.len()
    }

    /// Count of warm storage slots across all addresses. (test only)
    #[cfg(test)]
    pub fn warm_storage_count(&self) -> usize {
        self.warm_storage.values().map(|set| set.len()).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{address, U256};

    #[test]
    fn test_address_warming() {
        let mut cache = PersistentWarmCache::new();
        let addr = address!("0x1234567890123456789012345678901234567890");

        assert!(!cache.is_address_warm(&addr));

        cache.warm_account_and_storage(addr, []);
        assert!(cache.is_address_warm(&addr));
    }

    #[test]
    fn test_storage_warming() {
        let mut cache = PersistentWarmCache::new();
        let addr = address!("0x1234567890123456789012345678901234567890");
        let key = U256::from(42);

        assert!(!cache.is_storage_warm(&addr, &key));

        cache.warm_account_and_storage(addr, [key]);
        assert!(cache.is_storage_warm(&addr, &key));

        let other_key = U256::from(99);
        assert!(!cache.is_storage_warm(&addr, &other_key));
    }

    #[test]
    fn test_multiple_slots() {
        let mut cache = PersistentWarmCache::new();
        let addr = address!("0x1234567890123456789012345678901234567890");

        cache.warm_account_and_storage(addr, [U256::from(1), U256::from(2), U256::from(3)]);

        assert_eq!(cache.warm_storage_count(), 3);
        assert!(cache.is_storage_warm(&addr, &U256::from(1)));
        assert!(cache.is_storage_warm(&addr, &U256::from(2)));
        assert!(cache.is_storage_warm(&addr, &U256::from(3)));
        assert!(!cache.is_storage_warm(&addr, &U256::from(4)));
    }

    #[test]
    fn test_multiple_addresses() {
        let mut cache = PersistentWarmCache::new();
        let addr1 = address!("0x1111111111111111111111111111111111111111");
        let addr2 = address!("0x2222222222222222222222222222222222222222");

        cache.warm_address(addr1);
        cache.warm_address(addr2);

        assert_eq!(cache.warm_address_count(), 2);
        assert!(cache.is_address_warm(&addr1));
        assert!(cache.is_address_warm(&addr2));
    }

    #[test]
    fn test_storage_isolation_between_addresses() {
        let mut cache = PersistentWarmCache::new();
        let addr1 = address!("0x1111111111111111111111111111111111111111");
        let addr2 = address!("0x2222222222222222222222222222222222222222");
        let key = U256::from(42);

        cache.warm_account_and_storage(addr1, [key]);

        assert!(cache.is_storage_warm(&addr1, &key));
        assert!(!cache.is_storage_warm(&addr2, &key));
    }
}
