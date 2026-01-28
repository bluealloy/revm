//! This module contains [`WarmAddresses`] struct that stores addresses that are warm loaded.
//!
//! It is used to optimize access to precompile addresses.

use context_interface::journaled_state::JournalLoadError;
use primitives::{Address, AddressMap, AddressSet, HashSet, StorageKey};

/// Bitmask for precompile addresses (0x01-0x3F).
/// All EVM implementations keep precompiles at low sequential addresses.
type PrecompileMask = u64;

/// Ethereum mainnet precompiles as a bitmask.
const ETH_PRECOMPILES: PrecompileMask = (1u64 << 1)  |  // 0x01: ECRecover
    (1u64 << 2)  |  // 0x02: SHA2-256
    (1u64 << 3)  |  // 0x03: RIPEMD-160
    (1u64 << 4)  |  // 0x04: Identity
    (1u64 << 5)  |  // 0x05: ModExp
    (1u64 << 6)  |  // 0x06: BN256Add
    (1u64 << 7)  |  // 0x07: BN256Mul
    (1u64 << 8)  |  // 0x08: BN256Pairing
    (1u64 << 9)  |  // 0x09: Blake2F
    (1u64 << 10); // 0x0a: Point evaluation

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
    /// Fast path: Precompiles at 0x00-0x3F (covers 99.9% of cases)
    precompiles_mask: u64,

    /// Slow path: Non-standard precompiles (if any)
    /// Only allocated if a chain actually has high-address precompiles
    extended_precompiles: Option<AddressSet>,
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
            precompiles_mask: ETH_PRECOMPILES,
            extended_precompiles: None,
            coinbase: None,
            access_list: AddressMap::default(),
        }
    }

    /// Returns the precompile addresses.
    #[inline]
    pub fn precompiles(&self) -> Vec<Address> {
        let mut addresses = Vec::new();

        unsafe {
            let ptr: *mut Address = addresses.as_mut_ptr();
            let mut idx = 0;

            // Iterate through the bitmask
            for i in 0..64 {
                if (self.precompiles_mask & (1 << i)) != 0 {
                    let mut addr = [0u8; 20];
                    addr[19] = i as u8;
                    ptr.add(idx).write(Address::from(addr));
                    idx += 1;
                }
            }
            addresses.set_len(idx);
        }

        // Add extended precompiles if any
        if let Some(ref extended) = self.extended_precompiles {
            addresses.extend(extended.iter().copied());
        }

        addresses
    }

    /// Returns the coinbase address.
    #[inline]
    pub fn coinbase(&self) -> Option<Address> {
        self.coinbase
    }

    /// Set the precompile addresses and short addresses.
    #[inline]
    pub fn set_precompile_addresses(&mut self, addresses: AddressSet) {
        // Reset state
        self.precompiles_mask = 0;
        self.extended_precompiles = None;

        for address in addresses {
            // Check if it fits in the bitmask (0x00-0x3F)(maybe this should be extended idk)
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
    pub fn is_short_precompile(&self, address: &Address) -> bool {
        // Fast path: Check if address is in the 0x00-0x3F range
        if address[..19] == [0u8; 19] {
            let a = address[19] as u64;
            if a < 64 && (self.precompiles_mask & (1 << a)) != 0 {
                return true;
            }
        }
        false
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
            .is_some_and(|set| set.contains(address))
    }

    /// Add an extended precompile at a non-standard address.
    #[inline]
    pub fn add_extended_precompile(&mut self, address: Address) {
        self.extended_precompiles
            .get_or_insert_with(AddressSet::default)
            .insert(address);
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
        if let Some(access_list) = self.access_list.get(address) {
            return access_list.contains(key);
        }

        false
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
    fn test_coinbase_management() {
        let mut warm_addresses = WarmAddresses::new();
        let coinbase_addr = address!("1234567890123456789012345678901234567890");

        // Test setting coinbase
        warm_addresses.set_coinbase(coinbase_addr);
        assert_eq!(warm_addresses.coinbase, Some(coinbase_addr));
        assert!(warm_addresses.is_warm(&coinbase_addr));

        // Test clearing coinbase
        warm_addresses.clear_coinbase_and_access_list();
        assert!(warm_addresses.coinbase.is_none());
        assert!(!warm_addresses.is_warm(&coinbase_addr));
    }

    #[test]
    fn test_short_address_precompiles() {
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
    fn test_regular_address_precompiles() {
        let mut warm = WarmAddresses::new();

        // Before adding extended precompile, high address should be cold
        let high_addr = address!("1234567890123456789012345678901234567890");
        assert!(!warm.is_warm(&high_addr));
        assert!(warm.extended_precompiles.is_none()); // No allocation yet

        // Add extended precompile
        warm.add_extended_precompile(high_addr);
        assert!(warm.is_warm(&high_addr));
        assert!(warm.extended_precompiles.is_some());

        // Standard precompiles still work
        let mut std_addr = [0u8; 20];
        std_addr[19] = 1;
        assert!(warm.is_warm(&Address::from(std_addr)));
    }

    #[test]
    fn test_mixed_address_types() {
        let mut warm_addresses = WarmAddresses::new();

        let mut short_bytes = [0u8; 20];
        short_bytes[19] = 7u8;
        let short_addr = Address::from(short_bytes);
        let regular_addr = address!("1234567890123456789012345678901234567890");

        let mut precompiles = HashSet::default();
        precompiles.insert(short_addr);
        precompiles.insert(regular_addr);

        warm_addresses.set_precompile_addresses(precompiles);

        // Both types should be warm
        assert!(warm_addresses.is_warm(&short_addr));
        assert!(warm_addresses.is_warm(&regular_addr));

        // Verify short address optimization is used
        assert!(warm_addresses.is_short_precompile(&short_addr));
    }
}
