//! This module contains [`WarmAddresses`] struct that stores addresses that are warm loaded.
//!
//! It is used to optimize access to precompile addresses.

use bitvec::{bitvec, vec::BitVec};
use context_interface::journaled_state::JournalLoadError;
use primitives::{
    short_address, Address, AddressMap, HashMap, HashSet, StorageKey, SHORT_ADDRESS_CAP,
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
    precompile_set: HashSet<Address>,
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
            precompile_set: HashSet::default(),
            precompile_short_addresses: bitvec![0; SHORT_ADDRESS_CAP],
            precompile_all_short_addresses: true,
            coinbase: None,
            access_list: HashMap::default(),
        }
    }

    /// Returns the precompile addresses.
    #[inline]
    pub fn precompiles(&self) -> &HashSet<Address> {
        &self.precompile_set
    }

    /// Returns the coinbase address.
    #[inline]
    pub fn coinbase(&self) -> Option<Address> {
        self.coinbase
    }

    /// Set the precompile addresses and short addresses.
    #[inline]
    pub fn set_precompile_addresses(&mut self, addresses: HashSet<Address>) {
        self.precompile_short_addresses.fill(false);

        let mut all_short_addresses = true;
        for address in addresses.iter() {
            if let Some(short_address) = short_address(address) {
                self.precompile_short_addresses.set(short_address, true);
            } else {
                all_short_addresses = false;
            }
        }

        self.precompile_all_short_addresses = all_short_addresses;
        self.precompile_set = addresses;
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

    /// Returns true if the address is warm loaded.
    #[inline]
    pub fn is_warm(&self, address: &Address) -> bool {
        // check if it is coinbase
        if Some(*address) == self.coinbase {
            return true;
        }

        // if it is part of access list.
        if self.access_list.contains_key(address) {
            return true;
        }

        // if there are no precompiles, it is cold loaded and bitvec is not set.
        if self.precompile_set.is_empty() {
            return false;
        }

        // check if it is short precompile address
        if let Some(short_address) = short_address(address) {
            return self.precompile_short_addresses[short_address];
        }

        if !self.precompile_all_short_addresses {
            // in the end check if it is inside precompile set
            return self.precompile_set.contains(address);
        }

        false
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
        let warm_addresses = WarmAddresses::new();
        assert!(warm_addresses.precompile_set.is_empty());
        assert_eq!(
            warm_addresses.precompile_short_addresses.len(),
            SHORT_ADDRESS_CAP
        );
        assert!(!warm_addresses.precompile_short_addresses.any());
        assert!(warm_addresses.coinbase.is_none());

        // Test Default trait
        let default_addresses = WarmAddresses::default();
        assert_eq!(warm_addresses, default_addresses);
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
        let mut warm_addresses = WarmAddresses::new();

        // Create short addresses (18 leading zeros, last 2 bytes < 300)
        let mut bytes1 = [0u8; 20];
        bytes1[19] = 1u8;
        let short_addr1 = Address::from(bytes1);

        let mut bytes2 = [0u8; 20];
        bytes2[19] = 5u8;
        let short_addr2 = Address::from(bytes2);

        let mut precompiles = HashSet::default();
        precompiles.insert(short_addr1);
        precompiles.insert(short_addr2);

        warm_addresses.set_precompile_addresses(precompiles.clone());

        // Verify storage
        assert_eq!(warm_addresses.precompile_set, precompiles);
        assert_eq!(
            warm_addresses.precompile_short_addresses.len(),
            SHORT_ADDRESS_CAP
        );

        // Verify bitvec optimization
        assert!(warm_addresses.precompile_short_addresses[1]);
        assert!(warm_addresses.precompile_short_addresses[5]);
        assert!(!warm_addresses.precompile_short_addresses[0]);

        // Verify warmth detection
        assert!(warm_addresses.is_warm(&short_addr1));
        assert!(warm_addresses.is_warm(&short_addr2));

        // Test non-existent short address
        let mut other_bytes = [0u8; 20];
        other_bytes[19] = 20u8;
        let other_short_addr = Address::from(other_bytes);
        assert!(!warm_addresses.is_warm(&other_short_addr));
    }

    #[test]
    fn test_regular_address_precompiles() {
        let mut warm_addresses = WarmAddresses::new();

        // Create non-short addresses
        let regular_addr = address!("1234567890123456789012345678901234567890");
        let mut bytes = [0u8; 20];
        bytes[18] = 1u8;
        bytes[19] = 44u8; // 300
        let boundary_addr = Address::from(bytes);

        let mut precompiles = HashSet::default();
        precompiles.insert(regular_addr);
        precompiles.insert(boundary_addr);

        warm_addresses.set_precompile_addresses(precompiles.clone());

        // Verify storage
        assert_eq!(warm_addresses.precompile_set, precompiles);
        assert!(!warm_addresses.precompile_short_addresses.any());

        // Verify warmth detection
        assert!(warm_addresses.is_warm(&regular_addr));
        assert!(warm_addresses.is_warm(&boundary_addr));

        // Test non-existent regular address
        let other_addr = address!("0987654321098765432109876543210987654321");
        assert!(!warm_addresses.is_warm(&other_addr));
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
        assert!(warm_addresses.precompile_short_addresses[7]);
        assert!(!warm_addresses.precompile_short_addresses[8]);
    }

    #[test]
    fn test_short_address_boundary() {
        let mut warm_addresses = WarmAddresses::new();

        // Address at boundary (SHORT_ADDRESS_CAP - 1)
        let mut boundary_bytes = [0u8; 20];
        let boundary_val = (SHORT_ADDRESS_CAP - 1) as u16;
        boundary_bytes[18] = (boundary_val >> 8) as u8;
        boundary_bytes[19] = boundary_val as u8;
        let boundary_addr = Address::from(boundary_bytes);

        let mut precompiles = HashSet::default();
        precompiles.insert(boundary_addr);

        warm_addresses.set_precompile_addresses(precompiles);

        assert!(warm_addresses.is_warm(&boundary_addr));
        assert!(warm_addresses.precompile_short_addresses[SHORT_ADDRESS_CAP - 1]);
    }
}
