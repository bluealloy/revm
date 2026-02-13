//! Ethereum precompiles implementation with bitmask and lookup table for O(1) access.
//! Goes to 128 because of Fjord
use crate::OnceLock;

use primitives::{Address, AddressMap};

use crate::{
    blake2, bls12_381, bn254, hash, identity, kzg_point_evaluation, modexp, secp256k1, secp256r1,
    PrecompileFn, PrecompileResult, PrecompileSpecId,
};

/// Type alias for precompile bitmask
type PrecompileMask = u128;

/// Maximum precompile address that can fit in the lookup table (0x3f = 63)
const MAX_PRECOMPILE_INDEX: usize = 128;

/// Ethereum mainnet precompiles as a bitmask for Homestead spec.
const HOMESTEAD_PRECOMPILES: PrecompileMask = (1u128 << 1)  |  // 0x01: ECRecover
    (1u128 << 2)  |  // 0x02: SHA2-256
    (1u128 << 3)  |  // 0x03: RIPEMD-160
    (1u128 << 4); // 0x04: Identity

/// Byzantium precompiles (includes Homestead + new ones)
const BYZANTIUM_PRECOMPILES: PrecompileMask = HOMESTEAD_PRECOMPILES |
    (1u128 << 5)  |  // 0x05: ModExp
    (1u128 << 6)  |  // 0x06: BN256Add
    (1u128 << 7)  |  // 0x07: BN256Mul
    (1u128 << 8); // 0x08: BN256Pairing

/// Istanbul precompiles (includes Byzantium + new ones)
const ISTANBUL_PRECOMPILES: PrecompileMask = BYZANTIUM_PRECOMPILES | (1u128 << 9); // 0x09: Blake2F

/// Berlin precompiles (same as Istanbul, but ModExp gas cost changed)
const BERLIN_PRECOMPILES: PrecompileMask = ISTANBUL_PRECOMPILES;

/// Cancun precompiles (includes Berlin + new ones)
const CANCUN_PRECOMPILES: PrecompileMask = BERLIN_PRECOMPILES | (1u128 << 10); // 0x0a: Point evaluation

/// Prague precompiles (includes Cancun + BLS12-381 operations)
const PRAGUE_PRECOMPILES: PrecompileMask = CANCUN_PRECOMPILES |
    (1u128 << 0x0b)  |  // 0x0b: BLS12_G1ADD
    (1u128 << 0x0c)  |  // 0x0c: BLS12_G1MSM
    (1u128 << 0x0d)  |  // 0x0d: BLS12_G2ADD
    (1u128 << 0x0e)  |  // 0x0e: BLS12_G2MSM
    (1u128 << 0x0f)  |  // 0x0f: BLS12_PAIRING_CHECK
    (1u128 << 0x10)  |  // 0x10: BLS12_MAP_FP_TO_G1
    (1u128 << 0x11); // 0x11: BLS12_MAP_FP2_TO_G2

/// Osaka precompiles (includes Prague + updated ModExp and P256Verify)
/// Note: P256VERIFY is at a higher address, so we still use the lookup table for standard precompiles
const OSAKA_PRECOMPILES: PrecompileMask = PRAGUE_PRECOMPILES;

/// Precompiles use a bitmask for membership testing and a lookup table for O(1) execution.
#[derive(Clone, Debug)]
pub struct Precompiles {
    /// Bitmask for Ethereum precompiles (addresses 0x01-0x3f can fit in u64)
    eth_precompile_addresses: PrecompileMask,
    /// Lookup table indexed by address last byte for O(1) access.
    /// Use `precompile_fns[address_byte] = Some(function)` when a precompile exists.
    precompile_fns: [Option<PrecompileFn>; MAX_PRECOMPILE_INDEX],
    /// Extended precompiles at higher addresses (e.g., P256VERIFY at 0x0100).
    extended_precompile_fns: AddressMap<PrecompileFn>,
}

impl Default for Precompiles {
    fn default() -> Self {
        Self {
            eth_precompile_addresses: 0_u128,
            precompile_fns: [None; MAX_PRECOMPILE_INDEX],
            extended_precompile_fns: AddressMap::default(),
        }
    }
}

impl Precompiles {
    /// Returns the precompiles for the given spec.
    pub fn new(spec: PrecompileSpecId) -> &'static Self {
        match spec {
            PrecompileSpecId::HOMESTEAD => Self::homestead(),
            PrecompileSpecId::BYZANTIUM => Self::byzantium(),
            PrecompileSpecId::ISTANBUL => Self::istanbul(),
            PrecompileSpecId::BERLIN => Self::berlin(),
            PrecompileSpecId::CANCUN => Self::cancun(),
            PrecompileSpecId::PRAGUE => Self::prague(),
            PrecompileSpecId::OSAKA => Self::osaka(),
        }
    }

    /// Returns precompiles for Homestead spec.
    pub fn homestead() -> &'static Self {
        static INSTANCE: OnceLock<Precompiles> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            let mut precompiles = Precompiles {
                eth_precompile_addresses: HOMESTEAD_PRECOMPILES,
                ..Default::default()
            };
            precompiles.set(1, secp256k1::ec_recover_run);
            precompiles.set(2, hash::sha256_run);
            precompiles.set(3, hash::ripemd160_run);
            precompiles.set(4, identity::identity_run);
            precompiles
        })
    }

    /// Returns precompiles for Byzantium spec.
    pub fn byzantium() -> &'static Self {
        static INSTANCE: OnceLock<Precompiles> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            let mut precompiles = Self::homestead().clone();
            precompiles.eth_precompile_addresses = BYZANTIUM_PRECOMPILES;
            // EIP-198: Big integer modular exponentiation.
            precompiles.set(5, modexp::byzantium_run);
            // EIP-196/197: bn254 operations
            precompiles.set(6, *bn254::add::BYZANTIUM.precompile());
            precompiles.set(7, *bn254::mul::BYZANTIUM.precompile());
            precompiles.set(8, *bn254::pair::BYZANTIUM.precompile());
            precompiles
        })
    }

    /// Returns precompiles for Istanbul spec.
    pub fn istanbul() -> &'static Self {
        static INSTANCE: OnceLock<Precompiles> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            let mut precompiles = Self::byzantium().clone();
            precompiles.eth_precompile_addresses = ISTANBUL_PRECOMPILES;
            // EIP-1108: Reduce alt_bn128 precompile gas costs.
            precompiles.set(6, *bn254::add::ISTANBUL.precompile());
            precompiles.set(7, *bn254::mul::ISTANBUL.precompile());
            precompiles.set(8, *bn254::pair::ISTANBUL.precompile());
            // EIP-152: Add BLAKE2 compression function `F` precompile.
            precompiles.set(9, blake2::run);
            precompiles
        })
    }

    /// Returns precompiles for Berlin spec.
    pub fn berlin() -> &'static Self {
        static INSTANCE: OnceLock<Precompiles> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            let mut precompiles = Self::istanbul().clone();
            precompiles.eth_precompile_addresses = BERLIN_PRECOMPILES;
            // EIP-2565: ModExp Gas Cost.
            precompiles.set(5, modexp::berlin_run);
            precompiles
        })
    }

    /// Returns precompiles for Cancun spec.
    ///
    /// If the `c-kzg` feature is not enabled, KZG Point Evaluation precompile will not be included,
    /// effectively making this the same as Berlin.
    pub fn cancun() -> &'static Self {
        static INSTANCE: OnceLock<Precompiles> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            let mut precompiles = Self::berlin().clone();
            precompiles.eth_precompile_addresses = CANCUN_PRECOMPILES;
            // EIP-4844: Shard Blob Transactions
            precompiles.set(0x0a, kzg_point_evaluation::run);
            precompiles
        })
    }

    /// Returns precompiles for Prague spec.
    pub fn prague() -> &'static Self {
        static INSTANCE: OnceLock<Precompiles> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            let mut precompiles = Self::cancun().clone();
            precompiles.eth_precompile_addresses = PRAGUE_PRECOMPILES;
            // EIP-2537: BLS12-381 operations
            precompiles.set(0x0b, bls12_381::g1_add::g1_add);
            precompiles.set(0x0c, bls12_381::g1_msm::g1_msm);
            precompiles.set(0x0d, bls12_381::g2_add::g2_add);
            precompiles.set(0x0e, bls12_381::g2_msm::g2_msm);
            precompiles.set(0x0f, bls12_381::pairing::pairing);
            precompiles.set(0x10, bls12_381::map_fp_to_g1::map_fp_to_g1);
            precompiles.set(0x11, bls12_381::map_fp2_to_g2::map_fp2_to_g2);
            precompiles
        })
    }

    /// Returns precompiles for Osaka spec.
    pub fn osaka() -> &'static Self {
        static INSTANCE: OnceLock<Precompiles> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            let mut precompiles = Self::prague().clone();
            precompiles.eth_precompile_addresses = OSAKA_PRECOMPILES;
            // Update ModExp with new gas costs
            precompiles.set(5, *modexp::OSAKA.precompile());
            // P256VERIFY lives at 0x0100 (address 256), which doesn't fit in the lookup table.
            precompiles.set_extended(
                u64_to_address(secp256r1::P256VERIFY_ADDRESS),
                secp256r1::p256_verify_osaka,
            );
            precompiles
        })
    }

    /// Returns the precompiles for the latest spec.
    pub fn latest() -> &'static Self {
        Self::osaka()
    }

    /// Set a precompile function at the given index.
    #[inline]
    pub fn set(&mut self, index: usize, fun: PrecompileFn) {
        if index < MAX_PRECOMPILE_INDEX {
            self.precompile_fns[index] = Some(fun);
            self.eth_precompile_addresses |= 1u128 << index;
        }
    }

    /// Set a precompile function at a non-standard address.
    #[inline]
    pub fn set_extended(&mut self, address: Address, fun: PrecompileFn) {
        self.extended_precompile_fns.insert(address, fun);
    }

    /// Checks if the given address is a precompile.
    /// Uses bitmask for O(1) lookup.
    #[inline]
    pub fn contains(&self, address: &Address) -> bool {
        if let Some(index) = address_to_index(address) {
            return (self.eth_precompile_addresses & (1u128 << index)) != 0;
        }
        self.extended_precompile_fns.contains_key(address)
    }

    /// Executes the precompile at the given address with O(1) lookup.
    #[inline]
    pub fn call(
        &self,
        address: &Address,
        input: &[u8],
        gas_limit: u64,
    ) -> Option<PrecompileResult> {
        if let Some(index) = address_to_index(address) {
            let fun = self.precompile_fns[index]?;
            return Some(fun(input, gas_limit));
        }
        let fun = self.extended_precompile_fns.get(address)?;
        Some(fun(input, gas_limit))
    }

    /// Returns the precompile function for the given address.
    #[inline]
    pub fn get(&self, address: &Address) -> Option<PrecompileFn> {
        if let Some(index) = address_to_index(address) {
            return self.precompile_fns[index];
        }
        self.extended_precompile_fns.get(address).copied()
    }

    /// Returns the number of precompiles.
    #[inline]
    pub fn len(&self) -> usize {
        self.eth_precompile_addresses.count_ones() as usize + self.extended_precompile_fns.len()
    }

    /// Checks if the precompiles list is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.eth_precompile_addresses == 0 && self.extended_precompile_fns.is_empty()
    }

    /// Returns an iterator over precompile addresses.
    pub fn addresses(&self) -> impl Iterator<Item = Address> + '_ {
        PrecompileAddressIterator {
            precompiles: self,
            current: 1,
        }
        .chain(self.extended_precompile_fns.keys().copied())
    }

    /// Returns the complement of `other` in `self`.
    /// Two entries are considered equal if the precompile addresses are equal.
    pub fn difference(&self, other: &Self) -> Self {
        let mut result = Precompiles::default();

        // Iterate through our precompiles and add those not in other
        for i in 1..MAX_PRECOMPILE_INDEX {
            if (self.eth_precompile_addresses & (1u128 << i)) != 0
                && (other.eth_precompile_addresses & (1u128 << i)) == 0
            {
                if let Some(fun) = self.precompile_fns[i] {
                    result.precompile_fns[i] = Some(fun);
                    result.eth_precompile_addresses |= 1u128 << i;
                }
            }
        }
        for (addr, fun) in self.extended_precompile_fns.iter() {
            if !other.extended_precompile_fns.contains_key(addr) {
                result.extended_precompile_fns.insert(*addr, *fun);
            }
        }

        result
    }

    /// Returns the intersection of `self` and `other`.
    /// Two entries are considered equal if the precompile addresses are equal.
    pub fn intersection(&self, other: &Self) -> Self {
        let mut result = Precompiles::default();

        // Iterate through and add precompiles present in both
        for i in 1..MAX_PRECOMPILE_INDEX {
            if (self.eth_precompile_addresses & (1u128 << i)) != 0
                && (other.eth_precompile_addresses & (1u128 << i)) != 0
            {
                // Use self's function (they should be the same if addresses match)
                if let Some(fun) = self.precompile_fns[i] {
                    result.precompile_fns[i] = Some(fun);
                    result.eth_precompile_addresses |= 1u128 << i;
                }
            }
        }
        for (addr, fun) in self.extended_precompile_fns.iter() {
            if other.extended_precompile_fns.contains_key(addr) {
                result.extended_precompile_fns.insert(*addr, *fun);
            }
        }

        result
    }
}

/// Converts a u64 to an Address by padding with zeros on the left.
///
/// Note that 12 bytes of zeros + 8 bytes from u64 = 20 bytes (address length).
/// This function is used as a convenience for specifying precompile addresses.
#[inline]
pub const fn u64_to_address(x: u64) -> Address {
    let x = x.to_be_bytes();
    Address::new([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, x[0], x[1], x[2], x[3], x[4], x[5], x[6], x[7],
    ])
}

/// Converts an address to an index for the lookup table.
///
/// Returns `Some(index)` if the address is in the form 0x00...00XX where XX is 1-63,
/// otherwise returns `None`.
#[inline]
pub fn address_to_index(address: &Address) -> Option<usize> {
    let bytes = address.as_slice();

    // Check if first 19 bytes are all zeros
    if bytes[..19].iter().all(|&b| b == 0) {
        let last_byte = bytes[19] as usize;
        // Only addresses 0x01-0x3f can fit in the lookup table
        if last_byte > 0 && last_byte < MAX_PRECOMPILE_INDEX {
            return Some(last_byte);
        }
    }
    None
}

/// Iterator over precompile addresses.
struct PrecompileAddressIterator<'a> {
    precompiles: &'a Precompiles,
    current: usize,
}

impl<'a> Iterator for PrecompileAddressIterator<'a> {
    type Item = Address;

    fn next(&mut self) -> Option<Self::Item> {
        while self.current < MAX_PRECOMPILE_INDEX {
            let idx = self.current;
            self.current += 1;

            if (self.precompiles.eth_precompile_addresses & (1u128 << idx)) != 0 {
                return Some(u64_to_address(idx as u64));
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.precompiles.len();
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for PrecompileAddressIterator<'a> {
    fn len(&self) -> usize {
        // Count remaining set bits from current position onward
        let mask = !((1u128 << self.current) - 1); // Mask for bits >= current
        (self.precompiles.eth_precompile_addresses & mask).count_ones() as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bitmask_contains() {
        let precompiles = Precompiles::homestead();

        // Homestead should have precompiles at 0x01-0x04
        assert!(precompiles.contains(&u64_to_address(1))); // ECRECOVER
        assert!(precompiles.contains(&u64_to_address(2))); // SHA256
        assert!(precompiles.contains(&u64_to_address(3))); // RIPEMD160
        assert!(precompiles.contains(&u64_to_address(4))); // Identity

        // Should not have later precompiles
        assert!(!precompiles.contains(&u64_to_address(5))); // ModExp (Byzantium)
        assert!(!precompiles.contains(&u64_to_address(100)));
    }

    #[test]
    fn test_call_execution() {
        let precompiles = Precompiles::homestead();

        // Should be able to call existing precompiles
        assert!(precompiles
            .call(&u64_to_address(1), &[], u64::MAX)
            .is_some());

        // Should return None for non-existent precompiles
        assert!(precompiles
            .call(&u64_to_address(5), &[], u64::MAX)
            .is_none());
    }

    #[test]
    fn test_bitmask_progression() {
        let homestead = Precompiles::homestead();
        let byzantium = Precompiles::byzantium();
        let istanbul = Precompiles::istanbul();

        // Check that bitmasks include previous specs
        assert_eq!(homestead.eth_precompile_addresses, HOMESTEAD_PRECOMPILES);
        assert_eq!(byzantium.eth_precompile_addresses, BYZANTIUM_PRECOMPILES);
        assert_eq!(istanbul.eth_precompile_addresses, ISTANBUL_PRECOMPILES);

        // Byzantium should include all Homestead precompiles
        assert!(byzantium.contains(&u64_to_address(1)));
        assert!(byzantium.contains(&u64_to_address(5))); // New in Byzantium
    }

    #[test]
    fn test_address_to_index() {
        assert_eq!(address_to_index(&u64_to_address(1)), Some(1));
        assert_eq!(address_to_index(&u64_to_address(10)), Some(10));
        assert_eq!(address_to_index(&u64_to_address(63)), Some(63));

        // Address 0 should return None (not a valid precompile)
        assert_eq!(address_to_index(&u64_to_address(0)), None);

        // Addresses >= 64 should return None (don't fit in lookup table)
        assert_eq!(address_to_index(&u64_to_address(129)), None);
        assert_eq!(address_to_index(&u64_to_address(130)), None);

        // Non-standard addresses should return None
        let non_standard = Address::new([1; 20]);
        assert_eq!(address_to_index(&non_standard), None);
    }

    #[test]
    fn test_len_and_empty() {
        let homestead = Precompiles::homestead();
        assert_eq!(homestead.len(), 4); // 4 precompiles in Homestead
        assert!(!homestead.is_empty());

        let empty = Precompiles::default();
        assert_eq!(empty.len(), 0);
        assert!(empty.is_empty());
    }

    #[test]
    fn test_addresses_iterator() {
        let homestead = Precompiles::homestead();
        let addresses: Vec<_> = homestead.addresses().collect();

        assert_eq!(addresses.len(), 4);
        assert!(addresses.contains(&u64_to_address(1)));
        assert!(addresses.contains(&u64_to_address(2)));
        assert!(addresses.contains(&u64_to_address(3)));
        assert!(addresses.contains(&u64_to_address(4)));
    }
}
