//! This module contains [`WarmAddresses`] struct that stores addresses that are warm loaded.
//!
//! It is used to optimize access to precompile addresses.

use context_interface::journaled_state::JournalLoadError;
use primitives::{
    short_address, Address, AddressMap, AddressSet, HashSet, StorageKey, SHORT_ADDRESS_CAP,
};

/// Number of bytes needed to hold SHORT_ADDRESS_CAP bits (300 bits == 38 bytes).
const PRECOMPILE_SHORT_ADDRESS_BYTES: usize = SHORT_ADDRESS_CAP.div_ceil(8);

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
    /// Fixed bitset of precompile short addresses (one bit per possible short address).
    /// Uses a plain byte array + manual bit ops for fast access in the hot path
    /// (avoids BitVec overhead, similar to JumpTable optimization).
    #[cfg_attr(feature = "serde", serde(with = "fixed_array"))]
    precompile_short_addresses: [u8; PRECOMPILE_SHORT_ADDRESS_BYTES],
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
            precompile_short_addresses: [0u8; PRECOMPILE_SHORT_ADDRESS_BYTES],
            precompile_all_short_addresses: true,
            coinbase: None,
            access_list: AddressMap::default(),
        }
    }

    /// Returns the precompile addresses.
    #[inline]
    pub const fn precompiles(&self) -> &AddressSet {
        &self.precompile_set
    }

    /// Returns the coinbase address.
    #[inline]
    pub const fn coinbase(&self) -> Option<Address> {
        self.coinbase
    }

    /// Set the precompile addresses and short addresses.
    pub fn set_precompile_addresses(&mut self, addresses: &AddressSet) {
        self.precompile_short_addresses.fill(0);

        let mut all_short_addresses = true;
        for address in addresses.iter() {
            if let Some(short_address) = short_address(address) {
                self.set_precompile_short(short_address);
            } else {
                all_short_addresses = false;
            }
        }

        self.precompile_all_short_addresses = all_short_addresses;
        self.precompile_set.clone_from(addresses);
    }

    /// Set the bit for a short precompile address.
    ///
    /// Uses manual bit manipulation on a fixed byte array (instead of `BitVec`)
    /// to avoid per-access overhead
    #[inline(always)]
    const fn set_precompile_short(&mut self, idx: usize) {
        debug_assert!(
            idx < SHORT_ADDRESS_CAP,
            "Index out of bounds for short address"
        );
        // get the index to the byte in the arr
        let byte = idx >> 3;
        // bit position within the byte (0..7)
        let bit = 1 << (idx & 7);
        self.precompile_short_addresses[byte] |= bit;
    }

    /// Returns whether the bit for the given short address index is set.
    ///
    /// This is the hot path inside `is_warm`/`is_cold` for short precompile
    /// addresses. We perform direct byte + bit operations on the fixed array
    /// instead of going through `BitVec` (same technique I used in `JumpTable::is_valid`).
    #[inline(always)]
    pub(crate) const fn is_precompile_short(&self, idx: usize) -> bool {
        debug_assert!(
            idx < SHORT_ADDRESS_CAP,
            "Index out of bounds for short address"
        );
        let byte = idx >> 3;
        let bit = 1 << (idx & 7);
        self.precompile_short_addresses[byte] & bit != 0
    }

    /// Set the coinbase address.
    #[inline]
    pub const fn set_coinbase(&mut self, address: Address) {
        self.coinbase = Some(address);
    }

    /// Set the access list.
    #[inline]
    pub fn set_access_list(&mut self, access_list: AddressMap<HashSet<StorageKey>>) {
        self.access_list = access_list;
    }

    /// Returns the access list.
    #[inline]
    pub const fn access_list(&self) -> &AddressMap<HashSet<StorageKey>> {
        &self.access_list
    }

    /// Clear the coinbase address.
    #[inline]
    pub const fn clear_coinbase(&mut self) {
        self.coinbase = None;
    }

    /// Clear the coinbase and access list.
    #[inline]
    pub fn clear_coinbase_and_access_list(&mut self) {
        self.coinbase = None;
        self.access_list.clear();
    }

    /// Returns true if the address is warm loaded.
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
            return self.is_precompile_short(short_address);
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

#[cfg(feature = "serde")]
mod fixed_array {
    extern crate alloc;
    use serde::de::{value::MapAccessDeserializer, Error, MapAccess, SeqAccess, Visitor};
    use serde::{Deserialize, Deserializer, Serializer};

    pub(super) fn serialize<S: Serializer, const N: usize>(
        arr: &[u8; N],

        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(arr)
    }

    #[derive(Deserialize)]

    struct LegacyBitVec {
        #[allow(dead_code)]
        order: serde::de::IgnoredAny,
        #[allow(dead_code)]
        head: LegacyHead,
        #[allow(dead_code)]
        bits: usize,
        data: alloc::vec::Vec<u8>,
    }

    #[derive(Deserialize)]

    struct LegacyHead {
        #[allow(dead_code)]
        width: usize,
        #[allow(dead_code)]
        index: usize,
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>, const N: usize>(
        deserializer: D,
    ) -> Result<[u8; N], D::Error> {
        struct FixedArrayVisitor<const N: usize>;

        impl<'de, const N: usize> Visitor<'de> for FixedArrayVisitor<N> {
            type Value = [u8; N];

            fn expecting(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "byte array or legacy BitVec")
            }

            fn visit_bytes<E: Error>(self, v: &[u8]) -> Result<[u8; N], E> {
                v.try_into()
                    .map_err(|_| E::custom("invalid byte array length"))
            }

            fn visit_borrowed_bytes<E: Error>(self, v: &'de [u8]) -> Result<[u8; N], E> {
                self.visit_bytes(v)
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<[u8; N], A::Error> {
                let mut arr = [0; N];

                for b in &mut arr {
                    *b = seq
                        .next_element()?
                        .ok_or_else(|| A::Error::custom("invalid byte array length"))?;
                }

                if seq.next_element::<serde::de::IgnoredAny>()?.is_some() {
                    return Err(A::Error::custom("invalid byte array length"));
                }

                Ok(arr)
            }

            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<[u8; N], A::Error> {
                let mut data = LegacyBitVec::deserialize(MapAccessDeserializer::new(map))?.data;

                data.resize(N, 0);

                data.try_into()
                    .map_err(|_| A::Error::custom("legacy BitVec exceeds fixed array size"))
            }
        }

        deserializer.deserialize_any(FixedArrayVisitor::<N>)
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
            PRECOMPILE_SHORT_ADDRESS_BYTES
        );
        assert!(warm_addresses
            .precompile_short_addresses
            .iter()
            .all(|&b| b == 0));
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

        warm_addresses.set_precompile_addresses(&precompiles);

        // Verify storage
        assert_eq!(warm_addresses.precompile_set, precompiles);
        assert_eq!(
            warm_addresses.precompile_short_addresses.len(),
            PRECOMPILE_SHORT_ADDRESS_BYTES
        );

        // Verify optimization (replaces old BitVec)
        assert!(warm_addresses.is_precompile_short(1));
        assert!(warm_addresses.is_precompile_short(5));
        assert!(!warm_addresses.is_precompile_short(0));

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

        warm_addresses.set_precompile_addresses(&precompiles);

        // Verify storage
        assert_eq!(warm_addresses.precompile_set, precompiles);
        assert!(warm_addresses
            .precompile_short_addresses
            .iter()
            .all(|&b| b == 0));

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

        warm_addresses.set_precompile_addresses(&precompiles);

        // Both types should be warm
        assert!(warm_addresses.is_warm(&short_addr));
        assert!(warm_addresses.is_warm(&regular_addr));

        // Verify short address optimization is used
        assert!(warm_addresses.is_precompile_short(7));
        assert!(!warm_addresses.is_precompile_short(8));
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

        warm_addresses.set_precompile_addresses(&precompiles);

        assert!(warm_addresses.is_warm(&boundary_addr));
        assert!(warm_addresses.is_precompile_short(SHORT_ADDRESS_CAP - 1));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_serde_roundtrip() {
        let mut warm_addresses = WarmAddresses::new();

        // Create short and non-short precompiles
        let mut short_bytes = [0u8; 20];
        short_bytes[19] = 1u8;
        let short_addr = Address::from(short_bytes);

        let regular_addr = address!("1234567890123456789012345678901234567890");

        let mut precompiles = HashSet::default();
        precompiles.insert(short_addr);
        precompiles.insert(regular_addr);

        warm_addresses.set_precompile_addresses(&precompiles);

        let coinbase_addr = address!("0000000000000000000000000000000000000001");
        warm_addresses.set_coinbase(coinbase_addr);

        // Set access list with a storage slot
        let mut access_list = AddressMap::default();
        let mut slots = HashSet::default();
        slots.insert(primitives::U256::from(42));
        access_list.insert(regular_addr, slots);
        warm_addresses.set_access_list(access_list);

        let serialized = serde_json::to_string(&warm_addresses).expect("Failed to serialize");

        let deserialized: WarmAddresses =
            serde_json::from_str(&serialized).expect("Failed to deserialize");

        assert_eq!(warm_addresses, deserialized);

        assert!(deserialized.is_warm(&short_addr));
        assert!(deserialized.is_warm(&regular_addr));
        assert!(deserialized.is_warm(&coinbase_addr));
    }

    #[cfg(feature = "serde")]
    #[test]
    fn test_warm_addresses_deserializes_legacy_bitvec_format() {
        use primitives::Address;

        let short_addr = Address::with_last_byte(1);
        let mut warm = WarmAddresses::new();
        let mut precompiles = AddressSet::default();
        precompiles.insert(short_addr);
        warm.set_precompile_addresses(&precompiles);

        assert!(warm.is_warm(&short_addr));

        let serialized = serde_json::to_string(&warm).unwrap();

        // Ensure we're not emitting legacy object form anymore
        assert!(!serialized.contains(r#""order""#));

        let modern: WarmAddresses = serde_json::from_str(&serialized).unwrap();

        assert!(modern.is_warm(&short_addr));

        // Historical BitVec format
        let legacy_json = r#"
    {
        "precompile_set":["0x0000000000000000000000000000000000000001"],
        "precompile_short_addresses":{
            "order":"bitvec::order::Lsb0",
            "head":{"width":8,"index":0},
            "bits":300,
            "data":[2,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0]
        },
        "precompile_all_short_addresses":true,
        "coinbase":null,
        "access_list":{}
    }
    "#;

        let legacy: WarmAddresses = serde_json::from_str(legacy_json).unwrap();
        assert!(legacy.is_warm(&short_addr));
        assert!(legacy.is_precompile_short(1));

        let reserialized = serde_json::to_string(&legacy).unwrap();
        assert!(!reserialized.contains(r#""order""#));
    }
}

#[cfg(test)]
mod bench_is_short_precompile {
    use super::*;
    use std::time::Instant;

    use bitvec::{bitvec, vec::BitVec};

    const ITERATIONS: usize = 1_000_000;
    const TEST_SIZE: usize = SHORT_ADDRESS_CAP;

    /// Legacy BitVec version.
    #[derive(Clone)]
    struct ShortAddressesWithBitVec {
        bits: BitVec,
    }

    impl ShortAddressesWithBitVec {
        fn new() -> Self {
            Self {
                bits: bitvec![0; SHORT_ADDRESS_CAP],
            }
        }

        fn set_many(&mut self, indices: impl IntoIterator<Item = usize>) {
            for i in indices {
                if i < SHORT_ADDRESS_CAP {
                    self.bits.set(i, true);
                }
            }
        }

        /// Old BitVec [] style (for comparison).
        #[inline]
        fn is_set(&self, idx: usize) -> bool {
            self.bits[idx]
        }
    }

    fn create_test_data() -> (WarmAddresses, ShortAddressesWithBitVec) {
        let mut real = WarmAddresses::new();
        let mut precompile_set = AddressSet::default();

        for i in (0..TEST_SIZE).step_by(3) {
            let mut bytes = [0u8; 20];
            bytes[18] = (i >> 8) as u8;
            bytes[19] = i as u8;
            precompile_set.insert(Address::from(bytes));
        }
        real.set_precompile_addresses(&precompile_set);

        let mut legacy = ShortAddressesWithBitVec::new();
        legacy.set_many((0..TEST_SIZE).step_by(3));

        (real, legacy)
    }

    fn benchmark_implementation<F>(name: &str, table: &F, test_fn: impl Fn(&F, usize) -> bool)
    where
        F: Clone,
    {
        for i in 0..10_000 {
            std::hint::black_box(test_fn(table, i % TEST_SIZE));
        }

        let start = Instant::now();
        let mut count = 0;

        for i in 0..ITERATIONS {
            if test_fn(table, i % TEST_SIZE) {
                count += 1;
            }
        }

        let duration = start.elapsed();
        let ns_per_op = duration.as_nanos() as f64 / ITERATIONS as f64;
        let ops_per_sec = ITERATIONS as f64 / duration.as_secs_f64();

        println!("{name} Performance:");
        println!("  Time per op: {ns_per_op:.2} ns");
        println!("  Ops per sec: {ops_per_sec:.0}");
        println!("  True count: {count}");
        println!();

        std::hint::black_box(count);
    }

    #[test]
    fn bench_is_short_precompile() {
        println!("\nWarmAddresses short precompile bit test Benchmark Comparison");
        println!("============================================================");

        let (real_warm, legacy_bitvec) = create_test_data();

        benchmark_implementation(
            "WarmAddresses (Fixed Array + Manual Bits)",
            &real_warm,
            |wa, idx| wa.is_precompile_short(idx),
        );

        benchmark_implementation("Legacy (BitVec indexing)", &legacy_bitvec, |lb, idx| {
            lb.is_set(idx)
        });

        println!("Benchmark completed successfully!\n");
    }

    #[test]
    fn bench_different_access_patterns() {
        let (real_warm, legacy_bitvec) = create_test_data();

        println!("Short Precompile Access Pattern Comparison");
        println!("==========================================");

        let start = Instant::now();
        for i in 0..ITERATIONS {
            std::hint::black_box(real_warm.is_precompile_short(i % TEST_SIZE));
        }
        let fixed_sequential = start.elapsed();

        let start = Instant::now();
        for i in 0..ITERATIONS {
            std::hint::black_box(legacy_bitvec.is_set(i % TEST_SIZE));
        }
        let bitvec_sequential = start.elapsed();

        let start = Instant::now();
        for i in 0..ITERATIONS {
            std::hint::black_box(real_warm.is_precompile_short((i * 17) % TEST_SIZE));
        }
        let fixed_random = start.elapsed();

        let start = Instant::now();
        for i in 0..ITERATIONS {
            std::hint::black_box(legacy_bitvec.is_set((i * 17) % TEST_SIZE));
        }
        let bitvec_random = start.elapsed();

        println!("Sequential Access:");
        println!(
            "  Fixed Array: {:.2} ns/op",
            fixed_sequential.as_nanos() as f64 / ITERATIONS as f64
        );
        println!(
            "  BitVec:      {:.2} ns/op",
            bitvec_sequential.as_nanos() as f64 / ITERATIONS as f64
        );
        println!(
            "  Speedup: {:.1}x",
            bitvec_sequential.as_nanos() as f64 / fixed_sequential.as_nanos() as f64
        );

        println!();
        println!("Random Access:");
        println!(
            "  Fixed Array: {:.2} ns/op",
            fixed_random.as_nanos() as f64 / ITERATIONS as f64
        );
        println!(
            "  BitVec:      {:.2} ns/op",
            bitvec_random.as_nanos() as f64 / ITERATIONS as f64
        );
        println!(
            "  Speedup: {:.1}x",
            bitvec_random.as_nanos() as f64 / fixed_random.as_nanos() as f64
        );
    }
}
