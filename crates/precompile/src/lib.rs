//! # revm-precompile
//!
//! Implementations of EVM precompiled contracts.
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
#[cfg(not(feature = "std"))]
extern crate alloc as std;

#[allow(unreachable_code)]
pub mod blake2;
pub mod bls12_381;
pub mod bls12_381_const;
pub mod bls12_381_utils;
pub mod bn254;
pub mod hash;
mod id;
pub mod identity;
pub mod interface;
pub mod kzg_point_evaluation;
pub mod modexp;
pub mod secp256k1;
pub mod secp256r1;
pub mod utilities;

pub use primitives;

pub use id::PrecompileId;
pub use interface::*;

use core::fmt::{self, Debug};

// silence arkworks lint as bn impl will be used as default if both are enabled.
cfg_if::cfg_if! {
    if #[cfg(feature = "bn")]{
        use ark_bn254 as _;
        use ark_ff as _;
        use ark_ec as _;
        use ark_serialize as _;
    }
}

use arrayref as _;

// silence arkworks-bls12-381 lint as blst will be used as default if both are enabled.
cfg_if::cfg_if! {
    if #[cfg(feature = "blst")]{
        use ark_bls12_381 as _;
        use ark_ff as _;
        use ark_ec as _;
        use ark_serialize as _;
    }
}

// silence aurora-engine-modexp if gmp is enabled
#[cfg(feature = "gmp")]
use aurora_engine_modexp as _;

// silence p256 lint as aws-lc-rs will be used if both are enabled.
#[cfg(feature = "p256-aws-lc-rs")]
use p256 as _;

use core::hash::Hash;
use primitives::{
    hardfork::SpecId, short_address, Address, AddressMap, AddressSet, HashMap, OnceLock,
    SHORT_ADDRESS_CAP,
};
use std::boxed::Box;

/// Calculate the linear cost of a precompile.
#[inline]
pub const fn calc_linear_cost(len: usize, base: u64, word: u64) -> u64 {
    (len as u64).div_ceil(32) * word + base
}

/// Calculate the linear cost of a precompile.
#[deprecated(note = "please use `calc_linear_cost` instead")]
pub const fn calc_linear_cost_u32(len: usize, base: u64, word: u64) -> u64 {
    calc_linear_cost(len, base, word)
}

/// Precompiles contain map of precompile addresses to functions and AddressSet of precompile addresses.
#[derive(Clone, Debug)]
pub struct Precompiles {
    /// Precompiles
    inner: AddressMap<Precompile>,
    /// Addresses of precompiles.
    addresses: AddressSet,
    /// Optimized addresses filter.
    optimized_access: Box<[Option<Precompile>; SHORT_ADDRESS_CAP]>,
}

impl Default for Precompiles {
    fn default() -> Self {
        Self {
            inner: HashMap::default(),
            addresses: AddressSet::default(),
            optimized_access: Box::new([const { None }; SHORT_ADDRESS_CAP]),
        }
    }
}

impl Precompiles {
    /// Returns the precompiles for the given spec.
    pub fn new(spec: PrecompileSpecId) -> &'static Self {
        static INSTANCES: [OnceLock<Precompiles>; PrecompileSpecId::NEXT as usize + 1] =
            [const { OnceLock::new() }; PrecompileSpecId::NEXT as usize + 1];
        INSTANCES[spec as usize].get_or_init(|| init_precompiles(spec))
    }

    /// Returns precompiles for Homestead spec.
    pub fn homestead() -> &'static Self {
        Self::new(PrecompileSpecId::HOMESTEAD)
    }

    /// Returns precompiles for Byzantium spec.
    pub fn byzantium() -> &'static Self {
        Self::new(PrecompileSpecId::BYZANTIUM)
    }

    /// Returns precompiles for Istanbul spec.
    pub fn istanbul() -> &'static Self {
        Self::new(PrecompileSpecId::ISTANBUL)
    }

    /// Returns precompiles for Berlin spec.
    pub fn berlin() -> &'static Self {
        Self::new(PrecompileSpecId::BERLIN)
    }

    /// Returns precompiles for Cancun spec.
    ///
    /// If the `c-kzg` feature is not enabled KZG Point Evaluation precompile will not be included,
    /// effectively making this the same as Berlin.
    pub fn cancun() -> &'static Self {
        Self::new(PrecompileSpecId::CANCUN)
    }

    /// Returns precompiles for Prague spec.
    pub fn prague() -> &'static Self {
        Self::new(PrecompileSpecId::PRAGUE)
    }

    /// Returns precompiles for Osaka spec.
    pub fn osaka() -> &'static Self {
        Self::new(PrecompileSpecId::OSAKA)
    }

    /// Returns the precompiles for the latest spec.
    pub fn latest() -> &'static Self {
        Self::new(PrecompileSpecId::NEXT)
    }

    /// Returns inner HashMap of precompiles.
    #[inline]
    pub const fn inner(&self) -> &AddressMap<Precompile> {
        &self.inner
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
    pub fn get(&self, address: &Address) -> Option<&Precompile> {
        if let Some(short_address) = short_address(address) {
            return self.optimized_access[short_address].as_ref();
        }
        self.inner.get(address)
    }

    /// Returns the precompile for the given address.
    #[inline]
    pub fn get_mut(&mut self, address: &Address) -> Option<&mut Precompile> {
        self.inner.get_mut(address)
    }

    /// Is the precompiles list empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Returns the number of precompiles.
    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns the precompiles addresses as a set.
    #[inline]
    pub const fn addresses_set(&self) -> &AddressSet {
        &self.addresses
    }

    /// Extends the precompiles with the given precompiles.
    ///
    /// Other precompiles with overwrite existing precompiles.
    pub fn extend(&mut self, other: impl IntoIterator<Item = Precompile>) {
        let iter = other.into_iter();
        let (lower, _) = iter.size_hint();
        self.addresses.reserve(lower);
        self.inner.reserve(lower);
        for item in iter {
            let address = *item.address();
            if let Some(short_idx) = short_address(&address) {
                self.optimized_access[short_idx] = Some(item.clone());
            }
            self.addresses.insert(address);
            self.inner.insert(address, item);
        }
    }

    /// Returns complement of `other` in `self`.
    ///
    /// Two entries are considered equal if the precompile addresses are equal.
    pub fn difference(&self, other: &Self) -> Self {
        let Self { inner, .. } = self;

        let inner = inner
            .iter()
            .filter(|(a, _)| !other.inner.contains_key(*a))
            .map(|(a, p)| (*a, p.clone()))
            .collect::<AddressMap<_>>();

        let mut precompiles = Self::default();
        precompiles.extend(inner.into_iter().map(|p| p.1));
        precompiles
    }

    /// Returns intersection of `self` and `other`.
    ///
    /// Two entries are considered equal if the precompile addresses are equal.
    pub fn intersection(&self, other: &Self) -> Self {
        let Self { inner, .. } = self;

        let inner = inner
            .iter()
            .filter(|(a, _)| other.inner.contains_key(*a))
            .map(|(a, p)| (*a, p.clone()))
            .collect::<AddressMap<_>>();

        let mut precompiles = Self::default();
        precompiles.extend(inner.into_iter().map(|p| p.1));
        precompiles
    }
}

fn init_precompiles(spec: PrecompileSpecId) -> Precompiles {
    use PrecompileSpecId::*;

    let mut precompiles = Precompiles::default();

    // Homestead
    precompiles.extend([
        secp256k1::ECRECOVER,
        hash::SHA256,
        hash::RIPEMD160,
        identity::FUN,
    ]);

    if spec.is_enabled_in(BYZANTIUM) {
        // EIP-196: Precompiled contracts for addition and scalar multiplication on the elliptic curve alt_bn128.
        // EIP-197: Precompiled contracts for optimal ate pairing check on the elliptic curve alt_bn128.
        // EIP-198: Big integer modular exponentiation.
        precompiles.extend([
            modexp::BYZANTIUM,
            bn254::add::BYZANTIUM,
            bn254::mul::BYZANTIUM,
            bn254::pair::BYZANTIUM,
        ]);
    }

    if spec.is_enabled_in(ISTANBUL) {
        // EIP-152: Add BLAKE2 compression function `F` precompile.
        // EIP-1108: Reduce alt_bn128 precompile gas costs.
        precompiles.extend([
            bn254::add::ISTANBUL,
            bn254::mul::ISTANBUL,
            bn254::pair::ISTANBUL,
            blake2::FUN,
        ]);
    }

    if spec.is_enabled_in(BERLIN) {
        // EIP-2565: ModExp Gas Cost.
        precompiles.extend([modexp::BERLIN]);
    }

    if spec.is_enabled_in(CANCUN) {
        // EIP-4844: Shard Blob Transactions.
        precompiles.extend([kzg_point_evaluation::POINT_EVALUATION]);
    }

    if spec.is_enabled_in(PRAGUE) {
        // EIP-2537: Precompile for BLS12-381 curve operations.
        precompiles.extend(bls12_381::precompiles());
    }

    if spec.is_enabled_in(OSAKA) {
        // EIP-7823: Set upper bounds for MODEXP.
        // EIP-7883: ModExp Gas Cost Increase.
        precompiles.extend([modexp::OSAKA, secp256r1::P256VERIFY_OSAKA]);
    }

    precompiles
}

/// Precompile wrapper for simple eth function that provides complex interface on execution.
#[derive(Clone)]
pub struct Precompile {
    /// Unique identifier.
    id: PrecompileId,
    /// Precompile address.
    address: Address,
    /// Precompile function.
    fn_: PrecompileFn,
}

impl Debug for Precompile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Precompile {{ id: {:?}, address: {:?} }}",
            self.id, self.address
        )
    }
}

impl From<(PrecompileId, Address, PrecompileFn)> for Precompile {
    fn from((id, address, fn_): (PrecompileId, Address, PrecompileFn)) -> Self {
        Precompile { id, address, fn_ }
    }
}

impl From<Precompile> for (PrecompileId, Address) {
    fn from(value: Precompile) -> Self {
        (value.id, value.address)
    }
}

impl Precompile {
    /// Create new precompile.
    pub const fn new(id: PrecompileId, address: Address, fn_: PrecompileFn) -> Self {
        Self { id, address, fn_ }
    }

    /// Returns reference to precompile identifier.
    #[inline]
    pub const fn id(&self) -> &PrecompileId {
        &self.id
    }

    /// Returns reference to address.
    #[inline]
    pub const fn address(&self) -> &Address {
        &self.address
    }

    /// Executes the precompile.
    ///
    /// Returns `Ok(PrecompileOutput)` on success or non-fatal halt,
    /// or `Err(PrecompileError)` for fatal/unrecoverable errors.
    #[inline]
    pub fn execute(&self, input: &[u8], gas_limit: u64, reservoir: u64) -> PrecompileResult {
        (self.fn_)(input, gas_limit, reservoir)
    }
}

/// Ethereum hardfork spec ids. Represents the specs where precompiles had a change.
#[repr(u8)]
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
    /// Osaka spec added changes to modexp precompile:
    /// * [`EIP-7823: Set upper bounds for MODEXP`](https://eips.ethereum.org/EIPS/eip-7823).
    /// * [`EIP-7883: ModExp Gas Cost Increase`](https://eips.ethereum.org/EIPS/eip-7883)
    OSAKA,
}

impl From<SpecId> for PrecompileSpecId {
    fn from(spec_id: SpecId) -> Self {
        Self::from_spec_id(spec_id)
    }
}

impl PrecompileSpecId {
    /// The latest known precompile spec. This may refer to a highly experimental hard fork
    /// that is not yet finalized or deployed on any network.
    ///
    /// **Warning**: This value will change between minor versions as new hard forks are added.
    /// Do not rely on it for stable behavior.
    #[doc(alias = "MAX")]
    pub const NEXT: Self = Self::OSAKA;

    /// Returns `true` if the given specification ID is enabled in this spec.
    #[inline]
    pub const fn is_enabled_in(self, other: Self) -> bool {
        self as u8 >= other as u8
    }

    /// Returns the appropriate precompile Spec for the primitive [SpecId].
    pub const fn from_spec_id(spec_id: SpecId) -> Self {
        use SpecId::*;
        match spec_id {
            FRONTIER | FRONTIER_THAWING | HOMESTEAD | DAO_FORK | TANGERINE | SPURIOUS_DRAGON => {
                Self::HOMESTEAD
            }
            BYZANTIUM | CONSTANTINOPLE | PETERSBURG => Self::BYZANTIUM,
            ISTANBUL | MUIR_GLACIER => Self::ISTANBUL,
            BERLIN | LONDON | ARROW_GLACIER | GRAY_GLACIER | MERGE | SHANGHAI => Self::BERLIN,
            CANCUN => Self::CANCUN,
            PRAGUE => Self::PRAGUE,
            OSAKA | AMSTERDAM => Self::OSAKA,
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
    use super::*;

    fn temp_precompile(_input: &[u8], _gas_limit: u64, reservoir: u64) -> PrecompileResult {
        Ok(PrecompileOutput::halt(PrecompileHalt::OutOfGas, reservoir))
    }

    #[test]
    fn test_optimized_access() {
        let mut precompiles = Precompiles::istanbul().clone();
        assert!(precompiles.optimized_access[9].is_some());
        assert!(precompiles.optimized_access[10].is_none());

        precompiles.extend([Precompile::new(
            PrecompileId::Custom("test".into()),
            u64_to_address(100),
            temp_precompile,
        )]);
        precompiles.extend([Precompile::new(
            PrecompileId::Custom("test".into()),
            u64_to_address(101),
            temp_precompile,
        )]);

        let output = precompiles.optimized_access[100]
            .as_ref()
            .unwrap()
            .execute(&[], u64::MAX, 0)
            .unwrap();
        assert!(matches!(
            output.status,
            PrecompileStatus::Halt(PrecompileHalt::OutOfGas)
        ));

        let output = precompiles
            .get(&Address::left_padding_from(&[101]))
            .unwrap()
            .execute(&[], u64::MAX, 0)
            .unwrap();
        assert!(matches!(
            output.status,
            PrecompileStatus::Halt(PrecompileHalt::OutOfGas)
        ));
    }

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
