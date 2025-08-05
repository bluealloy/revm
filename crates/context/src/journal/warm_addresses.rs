//! This module contains [`WarmAddresses`] struct that stores addresses that are warm loaded.
//!
//! It is used to optimize access to precompile addresses.

use bitvec::{bitvec, order::Lsb0, vec::BitVec};
use primitives::{short_address, Address, HashSet, SHORT_ADDRESS_CAP};

/// Stores addresses that are warm loaded. Contains precompiles and coinbase address.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WarmAddresses {
    /// Set of warm loaded precompile addresses.
    pub precompile_set: HashSet<Address>,
    /// Bit vector of short address access.
    pub precompile_short_addresses: BitVec,
    /// Coinbase address.
    pub coinbase: Option<Address>,
}

impl WarmAddresses {
    /// Create a new warm addresses instance.
    #[inline]
    pub fn new() -> Self {
        Self {
            precompile_set: HashSet::new(),
            precompile_short_addresses: BitVec::new(),
            coinbase: None,
        }
    }

    /// Set the precompile addresses and short addresses.
    #[inline]
    pub fn set_precompile_addresses(&mut self, addresses: HashSet<Address>) {
        // short address is always smaller than SHORT_ADDRESS_CAP
        self.precompile_short_addresses = bitvec![usize, Lsb0; 0; SHORT_ADDRESS_CAP];

        for address in addresses.iter() {
            if let Some(short_address) = short_address(address) {
                self.precompile_short_addresses.set(short_address, true);
            }
        }

        self.precompile_set = addresses;
    }

    /// Set the coinbase address.
    #[inline]
    pub fn set_coinbase(&mut self, address: Address) {
        self.coinbase = Some(address);
    }

    /// Clear the coinbase address.
    #[inline]
    pub fn clear_coinbase(&mut self) {
        self.coinbase = None;
    }

    /// Returns the precompile addresses.
    #[inline]
    pub fn precompiles(&self) -> &HashSet<Address> {
        &self.precompile_set
    }

    /// Returns true if the address is warm loaded.
    #[inline]
    pub fn is_warm(&self, address: &Address) -> bool {
        // check if it is coinbase
        if Some(*address) == self.coinbase {
            return true;
        }

        // check if it is short precompile address
        if let Some(short_address) = short_address(address) {
            return self.precompile_short_addresses[short_address];
        }

        // finaly check if it is precompile
        self.precompile_set.contains(address)
    }

    /// Returns true if the address is cold loaded.
    #[inline]
    pub fn is_cold(&self, address: &Address) -> bool {
        !self.is_warm(address)
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
        assert!(warm_addresses.precompile_short_addresses.is_empty());
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
        warm_addresses.clear_coinbase();
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

        let mut precompiles = HashSet::new();
        precompiles.insert(short_addr1);
        precompiles.insert(short_addr2);

        warm_addresses.set_precompile_addresses(precompiles.clone());

        // Verify storage
        assert_eq!(warm_addresses.precompile_set, precompiles);
        assert_eq!(warm_addresses.precompile_short_addresses.len(), SHORT_ADDRESS_CAP);

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

        let mut precompiles = HashSet::new();
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

        let mut precompiles = HashSet::new();
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

        let mut precompiles = HashSet::new();
        precompiles.insert(boundary_addr);

        warm_addresses.set_precompile_addresses(precompiles);

        assert!(warm_addresses.is_warm(&boundary_addr));
        assert!(warm_addresses.precompile_short_addresses[SHORT_ADDRESS_CAP - 1]);
    }
}
