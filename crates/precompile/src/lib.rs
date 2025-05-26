//! # revm-precompile
//!
//! Implementations of EVM precompiled contracts.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
#[cfg(not(feature = "std"))]
extern crate alloc as std;

pub mod blake2;
pub mod bls12_381;
pub mod bls12_381_const;
pub mod bls12_381_utils;
pub mod bn128;
pub mod hash;
pub mod identity;
pub mod interface;
#[cfg(any(feature = "c-kzg", feature = "kzg-rs"))]
pub mod kzg_point_evaluation;
pub mod modexp;
pub mod secp256k1;
#[cfg(feature = "secp256r1")]
pub mod secp256r1;
pub mod utilities;

pub use interface::*;

// silence arkworks lint as bn impl will be used as default if both are enabled.
cfg_if::cfg_if! {
    if #[cfg(feature = "bn")]{
        use ark_bn254 as _;
        use ark_ff as _;
        use ark_ec as _;
        use ark_serialize as _;
    }
}

#[cfg(all(feature = "c-kzg", feature = "kzg-rs"))]
// silence kzg-rs lint as c-kzg will be used as default if both are enabled.
use kzg_rs as _;

// silence arkworks-bls12-381 lint as blst will be used as default if both are enabled.
cfg_if::cfg_if! {
    if #[cfg(feature = "blst")]{
        use ark_bls12_381 as _;
        use ark_ff as _;
        use ark_ec as _;
        use ark_serialize as _;
    }
}

use cfg_if::cfg_if;
use core::hash::Hash;
use once_cell::race::OnceBox;
use primitives::{hardfork::SpecId, Address, HashMap, HashSet};
use std::{boxed::Box, vec::Vec};

/// Calculate the linear cost of a precompile.
pub fn calc_linear_cost_u32(len: usize, base: u64, word: u64) -> u64 {
    (len as u64).div_ceil(32) * word + base
}

/// Precompiles contain map of precompile addresses to functions and HashSet of precompile addresses.
#[derive(Clone, Default, Debug)]
pub struct Precompiles {
    /// Precompiles
    inner: HashMap<Address, PrecompileFn>,
    /// Addresses of precompile
    addresses: HashSet<Address>,
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
        }
    }

    /// Returns precompiles for Homestead spec.
    pub fn homestead() -> &'static Self {
        static INSTANCE: OnceBox<Precompiles> = OnceBox::new();
        INSTANCE.get_or_init(|| {
            let mut precompiles = Precompiles::default();
            precompiles.extend([
                secp256k1::ECRECOVER,
                hash::SHA256,
                hash::RIPEMD160,
                identity::FUN,
            ]);
            Box::new(precompiles)
        })
    }

    /// Returns inner HashMap of precompiles.
    pub fn inner(&self) -> &HashMap<Address, PrecompileFn> {
        &self.inner
    }

    /// Returns precompiles for Byzantium spec.
    pub fn byzantium() -> &'static Self {
        static INSTANCE: OnceBox<Precompiles> = OnceBox::new();
        INSTANCE.get_or_init(|| {
            let mut precompiles = Self::homestead().clone();
            precompiles.extend([
                // EIP-198: Big integer modular exponentiation.
                modexp::BYZANTIUM,
                // EIP-196: Precompiled contracts for addition and scalar multiplication on the elliptic curve alt_bn128.
                // EIP-197: Precompiled contracts for optimal ate pairing check on the elliptic curve alt_bn128.
                bn128::add::BYZANTIUM,
                bn128::mul::BYZANTIUM,
                bn128::pair::BYZANTIUM,
            ]);
            Box::new(precompiles)
        })
    }

    /// Returns precompiles for Istanbul spec.
    pub fn istanbul() -> &'static Self {
        static INSTANCE: OnceBox<Precompiles> = OnceBox::new();
        INSTANCE.get_or_init(|| {
            let mut precompiles = Self::byzantium().clone();
            precompiles.extend([
                // EIP-1108: Reduce alt_bn128 precompile gas costs.
                bn128::add::ISTANBUL,
                bn128::mul::ISTANBUL,
                bn128::pair::ISTANBUL,
                // EIP-152: Add BLAKE2 compression function `F` precompile.
                blake2::FUN,
            ]);
            Box::new(precompiles)
        })
    }

    /// Returns precompiles for Berlin spec.
    pub fn berlin() -> &'static Self {
        static INSTANCE: OnceBox<Precompiles> = OnceBox::new();
        INSTANCE.get_or_init(|| {
            let mut precompiles = Self::istanbul().clone();
            precompiles.extend([
                // EIP-2565: ModExp Gas Cost.
                modexp::BERLIN,
            ]);
            Box::new(precompiles)
        })
    }

    /// Returns precompiles for Cancun spec.
    ///
    /// If the `c-kzg` feature is not enabled KZG Point Evaluation precompile will not be included,
    /// effectively making this the same as Berlin.
    pub fn cancun() -> &'static Self {
        static INSTANCE: OnceBox<Precompiles> = OnceBox::new();
        INSTANCE.get_or_init(|| {
            let mut precompiles = Self::berlin().clone();

            // EIP-4844: Shard Blob Transactions
            cfg_if! {
                if #[cfg(any(feature = "c-kzg", feature = "kzg-rs"))] {
                    let precompile = kzg_point_evaluation::POINT_EVALUATION.clone();
                } else {
                    let precompile = PrecompileWithAddress(u64_to_address(0x0A), |_,_| Err(PrecompileError::Fatal("c-kzg feature is not enabled".into())));
                }
            }


            precompiles.extend([
                precompile,
            ]);

            Box::new(precompiles)
        })
    }

    /// Returns precompiles for Prague spec.
    pub fn prague() -> &'static Self {
        static INSTANCE: OnceBox<Precompiles> = OnceBox::new();
        INSTANCE.get_or_init(|| {
            let mut precompiles = Self::cancun().clone();
            precompiles.extend(bls12_381::precompiles());
            Box::new(precompiles)
        })
    }

    /// Returns the precompiles for the latest spec.
    pub fn latest() -> &'static Self {
        Self::prague()
    }

    /// Returns an iterator over the precompiles addresses.
    #[inline]
    pub fn addresses(&self) -> impl ExactSizeIterator<Item = &Address> {
        self.inner.keys()
    }

    /// Consumes the type and returns all precompile addresses.
    #[inline]
    pub fn into_addresses(self) -> impl ExactSizeIterator<Item = Address> {
        self.inner.into_keys()
    }

    /// Is the given address a precompile.
    #[inline]
    pub fn contains(&self, address: &Address) -> bool {
        self.inner.contains_key(address)
    }

    /// Returns the precompile for the given address.
    #[inline]
    pub fn get(&self, address: &Address) -> Option<&PrecompileFn> {
        self.inner.get(address)
    }

    /// Returns the precompile for the given address.
    #[inline]
    pub fn get_mut(&mut self, address: &Address) -> Option<&mut PrecompileFn> {
        self.inner.get_mut(address)
    }

    /// Is the precompiles list empty.
    pub fn is_empty(&self) -> bool {
        self.inner.len() == 0
    }

    /// Returns the number of precompiles.
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns the precompiles addresses as a set.
    pub fn addresses_set(&self) -> &HashSet<Address> {
        &self.addresses
    }

    /// Extends the precompiles with the given precompiles.
    ///
    /// Other precompiles with overwrite existing precompiles.
    #[inline]
    pub fn extend(&mut self, other: impl IntoIterator<Item = PrecompileWithAddress>) {
        let items: Vec<PrecompileWithAddress> = other.into_iter().collect::<Vec<_>>();
        self.addresses.extend(items.iter().map(|p| *p.address()));
        self.inner.extend(items.into_iter().map(|p| (p.0, p.1)));
    }

    /// Returns complement of `other` in `self`.
    ///
    /// Two entries are considered equal if the precompile addresses are equal.
    pub fn difference(&self, other: &Self) -> Self {
        let Self { inner, .. } = self;

        let inner = inner
            .iter()
            .filter(|(a, _)| !other.inner.contains_key(*a))
            .map(|(a, p)| (*a, *p))
            .collect::<HashMap<_, _>>();

        let addresses = inner.keys().cloned().collect::<HashSet<_>>();

        Self { inner, addresses }
    }

    /// Returns intersection of `self` and `other`.
    ///
    /// Two entries are considered equal if the precompile addresses are equal.
    pub fn intersection(&self, other: &Self) -> Self {
        let Self { inner, .. } = self;

        let inner = inner
            .iter()
            .filter(|(a, _)| other.inner.contains_key(*a))
            .map(|(a, p)| (*a, *p))
            .collect::<HashMap<_, _>>();

        let addresses = inner.keys().cloned().collect::<HashSet<_>>();

        Self { inner, addresses }
    }
}

/// Precompile with address and function.
#[derive(Clone, Debug)]
pub struct PrecompileWithAddress(pub Address, pub PrecompileFn);

impl From<(Address, PrecompileFn)> for PrecompileWithAddress {
    fn from(value: (Address, PrecompileFn)) -> Self {
        PrecompileWithAddress(value.0, value.1)
    }
}

impl From<PrecompileWithAddress> for (Address, PrecompileFn) {
    fn from(value: PrecompileWithAddress) -> Self {
        (value.0, value.1)
    }
}

impl PrecompileWithAddress {
    /// Returns reference of address.
    #[inline]
    pub fn address(&self) -> &Address {
        &self.0
    }

    /// Returns reference of precompile.
    #[inline]
    pub fn precompile(&self) -> &PrecompileFn {
        &self.1
    }
}

/// Ethereum hardfork spec ids. Represents the specs where precompiles had a change.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum PrecompileSpecId {
    /// Frontier spec.
    HOMESTEAD,
    /// Byzantium spec introduced
    /// * [EIP-198](https://eips.ethereum.org/EIPS/eip-198) a EIP-198: Big integer modular exponentiation (at 0x05 address).
    /// * [EIP-196](https://eips.ethereum.org/EIPS/eip-196) a bn_add (at 0x06 address) and bn_mul (at 0x07 address) precompile
    /// * [EIP-197](https://eips.ethereum.org/EIPS/eip-197) a bn_pair (at 0x08 address) precompile
    BYZANTIUM,
    /// Istanbul spec introduced
    /// * [`EIP-152: Add BLAKE2 compression function`](https://eips.ethereum.org/EIPS/eip-152) `F` precompile (at 0x09 address).
    /// * [`EIP-1108: Reduce alt_bn128 precompile gas costs`](https://eips.ethereum.org/EIPS/eip-1108). It reduced the
    ///   gas cost of the bn_add, bn_mul, and bn_pair precompiles.
    ISTANBUL,
    /// Berlin spec made a change to:
    /// * [`EIP-2565: ModExp Gas Cost`](https://eips.ethereum.org/EIPS/eip-2565). It changed the gas cost of the modexp precompile.
    BERLIN,
    /// Cancun spec added
    /// * [`EIP-4844: Shard Blob Transactions`](https://eips.ethereum.org/EIPS/eip-4844). It added the KZG point evaluation precompile (at 0x0A address).
    CANCUN,
    /// Prague spec added bls precompiles [`EIP-2537: Precompile for BLS12-381 curve operations`](https://eips.ethereum.org/EIPS/eip-2537).
    /// * `BLS12_G1ADD` at address 0x0b
    /// * `BLS12_G1MSM` at address 0x0c
    /// * `BLS12_G2ADD` at address 0x0d
    /// * `BLS12_G2MSM` at address 0x0e
    /// * `BLS12_PAIRING_CHECK` at address 0x0f
    /// * `BLS12_MAP_FP_TO_G1` at address 0x10
    /// * `BLS12_MAP_FP2_TO_G2` at address 0x11
    PRAGUE,
}

impl From<SpecId> for PrecompileSpecId {
    fn from(spec_id: SpecId) -> Self {
        Self::from_spec_id(spec_id)
    }
}

impl PrecompileSpecId {
    /// Returns the appropriate precompile Spec for the primitive [SpecId].
    pub const fn from_spec_id(spec_id: primitives::hardfork::SpecId) -> Self {
        use primitives::hardfork::SpecId::*;
        match spec_id {
            FRONTIER | FRONTIER_THAWING | HOMESTEAD | DAO_FORK | TANGERINE | SPURIOUS_DRAGON => {
                Self::HOMESTEAD
            }
            BYZANTIUM | CONSTANTINOPLE | PETERSBURG => Self::BYZANTIUM,
            ISTANBUL | MUIR_GLACIER => Self::ISTANBUL,
            BERLIN | LONDON | ARROW_GLACIER | GRAY_GLACIER | MERGE | SHANGHAI => Self::BERLIN,
            CANCUN => Self::CANCUN,
            PRAGUE | OSAKA => Self::PRAGUE,
        }
    }
}

/// Const function for making an address by concatenating the bytes from two given numbers.
///
/// Note that 32 + 128 = 160 = 20 bytes (the length of an address).
///
/// This function is used as a convenience for specifying the addresses of the various precompiles.
#[inline]
pub const fn u64_to_address(x: u64) -> Address {
    let x = x.to_be_bytes();
    Address::new([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, x[0], x[1], x[2], x[3], x[4], x[5], x[6], x[7],
    ])
}

#[cfg(test)]
mod test {
    use crate::Precompiles;

    #[test]
    fn test_difference_precompile_sets() {
        let difference = Precompiles::istanbul().difference(Precompiles::berlin());
        assert!(difference.is_empty());
    }

    #[test]
    fn test_intersection_precompile_sets() {
        let intersection = Precompiles::homestead().intersection(Precompiles::byzantium());

        assert_eq!(intersection.len(), 4)
    }
}
