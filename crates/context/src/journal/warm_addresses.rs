//! This module contains [`WarmAddresses`] struct that stores addresses that are warm loaded.
//!
//! It is used to optimize access to precompile addresses.

use context_interface::journaled_state::JournalLoadError;
use primitives::{
    short_address, Address, AddressMap, AddressSet, HashSet, StorageKey, SHORT_ADDRESS_CAP,
};

/// Stores addresses that are warm loaded. Contains precompiles and coinbase address.
///
/// It contains precompiles addresses that are not changed frequently and AccessList that
/// is changed per transaction.
///
/// [WarmAddresses::precompiles] will always contain all precompile addresses.
///
/// As precompiles addresses are usually very small, precompile_short_addresses will
/// contain bitset of shrunk precompile address.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WarmAddresses {
    /// Set of warm loaded precompile addresses.
    precompile_set: AddressSet,
    /// Bit vector of precompile short addresses. If address is shorter than [`SHORT_ADDRESS_CAP`] it
    /// will be stored in this bit vector for faster access.
    precompile_short_addresses: BitVec,
    /// `true` if all precompiles are short addresses.
    precompile_all_short_addresses: bool,
    /// Coinbase address.
    coinbase: Option<Address>,
    /// Access list
    access_list: AddressMap<HashSet<StorageKey>>,
}

impl Default for WarmAddresses {
    fn default() -> Self {
        Self::new()
    }
}

impl WarmAddresses {
    /// Create a new warm addresses instance.
    #[inline]
    pub fn new() -> Self {
        Self {
            precompile_set: AddressSet::default(),
            precompile_short_addresses: bitvec![0; SHORT_ADDRESS_CAP],
            precompile_all_short_addresses: true,
            coinbase: None,
            access_list: AddressMap::default(),
        }
    }

    /// Create with custom precompile mask.
    #[inline]
    pub fn precompiles(&self) -> &AddressSet {
        &self.precompile_set
    }

    /// Returns the precompile mask.
    #[inline]
    pub fn precompiles_mask(&self) -> PrecompileMask {
        self.precompiles_mask
    }

    /// Add an extended precompile at a non-standard address.
    #[inline]
    pub fn set_precompile_addresses(&mut self, addresses: AddressSet) {
        self.precompile_short_addresses.fill(false);

        let mut all_short_addresses = true;
        for address in addresses.iter() {
            if let Some(short_address) = short_address(address) {
                self.precompile_short_addresses.set(short_address, true);
            } else {
                all_short_addresses = false;
            }
        }

    /// Set multiple extended precompiles at once.
    #[inline]
    pub fn set_extended_precompiles(&mut self, addresses: HashSet<Address>) {
        if !addresses.is_empty() {
            self.extended_precompiles = Some(addresses);
        } else {
            self.extended_precompiles = None;
        }
    }

    /// Set the coinbase address.
    #[inline]
    pub fn set_coinbase(&mut self, address: Address) {
        self.coinbase = Some(address);
    }

    /// Set the access list.
    #[inline]
    pub fn set_access_list(&mut self, access_list: AddressMap<HashSet<StorageKey>>) {
        self.access_list = access_list;
    }

    /// Returns the access list.
    #[inline]
    pub fn access_list(&self) -> &AddressMap<HashSet<StorageKey>> {
        &self.access_list
    }

    /// Clear the coinbase address.
    #[inline]
    pub fn clear_coinbase(&mut self) {
        self.coinbase = None;
    }

    /// Clear the coinbase and access list.
    #[inline]
    pub fn clear_coinbase_and_access_list(&mut self) {
        self.coinbase = None;
        self.access_list.clear();
    }

    /// Check if address is a precompile.
    #[inline]
    fn is_precompile(&self, address: &Address) -> bool {
        // Fast path: Check if address is in the 0x00-0x3F range
        if address[..19] == [0u8; 19] {
            let a = address[19] as u64;
            if a < 64 && (self.precompiles_mask & (1 << a)) != 0 {
                return true;
            }
        }

        // Slow path: Check extended precompiles (only if they exist)
        self.extended_precompiles
            .as_ref()
            .map_or(false, |set| set.contains(address))
    }

     /// Set precompiles from a collection of addresses.
    /// 
    /// Automatically separates addresses into:
    /// - Bitmask: for addresses 0x00-0x3F (fast path)
    /// - Extended: for addresses outside that range (slow path)
    /// 
    /// This is the most flexible API - pass any collection of addresses
    /// and it will optimize storage automatically.
    pub fn set_precompiles(&mut self, addresses: impl IntoIterator<Item = Address>) {
        // Reset state
        self.precompiles_mask = 0;
        self.extended_precompiles = None;
        
        for address in addresses {
            // Check if it fits in the bitmask (0x00-0x3F)
            if address[..19] == [0u8; 19] {
                let a = address[19] as u64;
                if a < 64 {
                    // Fast path: set bit in mask
                    self.precompiles_mask |= 1 << a;
                    continue;
                }
            }
            
            self.extended_precompiles
                .get_or_insert_with(HashSet::default)
                .insert(address);
        }
    }

    /// Returns true if the address is warm loaded.
    #[inline]
    pub fn is_warm(&self, address: &Address) -> bool {
        // Check in order of likelihood:
        // 1. Precompiles (most common in practice)
        // 2. Coinbase (once per block)
        // 3. Access list (varies per transaction)
        self.is_precompile(address)
            || Some(*address) == self.coinbase
            || self.access_list.contains_key(address)
    }

    /// Returns true if the storage is warm loaded.
    #[inline]
    pub fn is_storage_warm(&self, address: &Address, key: &StorageKey) -> bool {
        self.access_list
            .get(address)
            .map_or(false, |keys| keys.contains(key))
    }

    /// Returns all precompile addresses as a Vec.
    pub fn all_precompile_addresses(&self) -> Vec<Address> {
        let mut addresses = Vec::new();

        // Iterate through the bitmask
        for i in 0..64 {
            if (self.precompiles_mask & (1 << i)) != 0 {
                let mut addr = [0u8; 20];
                addr[19] = i as u8;
                addresses.push(Address::from(addr));
            }
        }

        // Add extended precompiles if any
        if let Some(ref extended) = self.extended_precompiles {
            addresses.extend(extended.iter().copied());
        }

        addresses
    }

    /// Returns true if the address is cold loaded.
    #[inline]
    pub fn is_cold(&self, address: &Address) -> bool {
        !self.is_warm(address)
    }

    /// Checks if the address is cold loaded and returns an error if it is and skip_cold_load is true.
    #[inline(never)]
    pub fn check_is_cold<E>(
        &self,
        address: &Address,
        skip_cold_load: bool,
    ) -> Result<bool, JournalLoadError<E>> {
        let is_cold = self.is_cold(address);

        if is_cold && skip_cold_load {
            return Err(JournalLoadError::ColdLoadSkipped);
        }

        Ok(is_cold)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use primitives::{address, Address};

    #[test]
    fn test_initialization() {
        let warm = WarmAddresses::new();
        assert_eq!(warm.precompiles_mask, ETH_PRECOMPILES);
        assert!(warm.extended_precompiles.is_none());
        assert!(warm.coinbase.is_none());
        assert!(warm.access_list.is_empty());
    }

    #[test]
    fn test_standard_precompiles() {
        let warm = WarmAddresses::new();

        // Test all standard Ethereum precompiles (0x01-0x0a)
        for i in 1u8..=10 {
            let mut addr = [0u8; 20];
            addr[19] = i;
            let precompile = Address::from(addr);
            assert!(
                warm.is_warm(&precompile),
                "Precompile 0x{:02x} should be warm",
                i
            );
        }

        // Test non-precompile low address
        let mut addr = [0u8; 20];
        addr[19] = 11;
        assert!(!warm.is_warm(&Address::from(addr)));
    }

    #[test]
    fn test_extended_precompiles() {
        let mut warm = WarmAddresses::new();

        // Before adding extended precompile, high address should be cold
        let high_addr = address!("1234567890123456789012345678901234567890");
        assert!(!warm.is_warm(&high_addr));
        assert!(warm.extended_precompiles.is_none()); // No allocation yet

        // Add extended precompile
        warm.add_extended_precompile(high_addr);
        assert!(warm.is_warm(&high_addr));
        assert!(warm.extended_precompiles.is_some()); // Now allocated

        // Standard precompiles still work
        let mut std_addr = [0u8; 20];
        std_addr[19] = 1;
        assert!(warm.is_warm(&Address::from(std_addr)));
    }

    #[test]
    fn test_custom_mask_l2() {
        // Example: L2 with precompiles at 0x01-0x0a and 0x0b-0x0f
        let l2_mask = ETH_PRECOMPILES | (1 << 11) | (1 << 12) | (1 << 13) | (1 << 14) | (1 << 15);

        let warm = WarmAddresses::with_precompiles(l2_mask);

        // Check standard precompiles
        let mut addr = [0u8; 20];
        addr[19] = 5;
        assert!(warm.is_warm(&Address::from(addr)));

        // Check L2 custom precompiles
        addr[19] = 12;
        assert!(warm.is_warm(&Address::from(addr)));

        // Still no extended_precompiles allocation (all fit in mask)
        assert!(warm.extended_precompiles.is_none());
    }

    #[test]
    fn test_coinbase() {
        let mut warm = WarmAddresses::new();
        let coinbase = address!("1234567890123456789012345678901234567890");

        warm.set_coinbase(coinbase);
        assert!(warm.is_warm(&coinbase));

        warm.clear_coinbase();
        assert!(!warm.is_warm(&coinbase));
    }

    #[test]
    fn test_access_list() {
        let mut warm = WarmAddresses::new();
        let addr = address!("1234567890123456789012345678901234567890");

        let mut access_list = HashMap::default();
        access_list.insert(addr, HashSet::default());
        warm.set_access_list(access_list);

        assert!(warm.is_warm(&addr));
    }

    #[test]
    fn test_storage_warmth() {
        let mut warm = WarmAddresses::new();
        let addr = address!("1234567890123456789012345678901234567890");
        let key = primitives::U256::from(42);

        let mut keys = HashSet::default();
        keys.insert(key);

        let mut access_list = HashMap::default();
        access_list.insert(addr, keys);
        warm.set_access_list(access_list);

        assert!(warm.is_storage_warm(&addr, &key));
        assert!(!warm.is_storage_warm(&addr, &primitives::U256::from(43)));
    }

    #[test]
    fn test_zero_overhead_for_standard_chains() {
        let warm = WarmAddresses::new();

        // Verify no HashSet allocation for standard Ethereum
        assert!(warm.extended_precompiles.is_none());

        // Size check: should be minimal
        // precompiles_mask: 8 bytes
        // extended_precompiles: 8 bytes (Option discriminant + pointer)
        // coinbase: 24 bytes (Option + Address)
        // access_list: 48 bytes (HashMap overhead)
        // Total: ~88 bytes (way better than original with BitVec + HashSet)
    }
}
