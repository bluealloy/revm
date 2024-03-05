//! # revm-precompile
//!
//! Implementations of EVM precompiled contracts.
#![warn(rustdoc::all)]
#![warn(unused_crate_dependencies)]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
#[cfg(not(feature = "std"))]
extern crate alloc as std;

mod blake2;
mod bn128;
mod hash;
mod identity;
#[cfg(feature = "c-kzg")]
pub mod kzg_point_evaluation;
mod modexp;
mod secp256k1;
pub mod utilities;

use core::hash::Hash;
#[doc(hidden)]
pub use revm_primitives as primitives;
pub use revm_primitives::{
    precompile::{PrecompileError as Error, *},
    Address, Bytes, HashMap, Log, B256,
};
use std::vec::Vec;

pub fn calc_linear_cost_u32(len: usize, base: u64, word: u64) -> u64 {
    (len as u64 + 32 - 1) / 32 * word + base
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct PrecompileOutput {
    pub cost: u64,
    pub output: Vec<u8>,
    pub logs: Vec<Log>,
}

impl PrecompileOutput {
    pub fn without_logs(cost: u64, output: Vec<u8>) -> Self {
        Self {
            cost,
            output,
            logs: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct Precompiles<CTX, EXTCXT> {
    /// Precompiles.
    pub inner: HashMap<Address, Precompile<CTX, EXTCXT>>,
}

impl<CTX, EXTCTX> Clone for Precompiles<CTX, EXTCTX> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<CTX, EXTCXT> Default for Precompiles<CTX, EXTCXT> {
    fn default() -> Self {
        Self {
            inner: HashMap::default(),
        }
    }
}

impl<CTX, EXTCXT> Precompiles<CTX, EXTCXT> {
    /// Returns the precompiles for the given spec.
    pub fn new(spec: PrecompileSpecId) -> Self {
        match spec {
            PrecompileSpecId::HOMESTEAD => Self::homestead(),
            PrecompileSpecId::BYZANTIUM => Self::byzantium(),
            PrecompileSpecId::ISTANBUL => Self::istanbul(),
            PrecompileSpecId::BERLIN => Self::berlin(),
            PrecompileSpecId::CANCUN => Self::cancun(),
            PrecompileSpecId::LATEST => Self::latest(),
        }
    }

    /// Returns precompiles for Homestead spec.
    pub fn homestead() -> Self {
        let mut precompiles = Precompiles::default();
        precompiles.extend([
            secp256k1::ECRECOVER,
            hash::SHA256,
            hash::RIPEMD160,
            identity::FUN,
        ]);
        precompiles
    }

    /// Returns precompiles for Byzantium spec.
    pub fn byzantium() -> Self {
        let mut precompiles = Precompiles::default();
        precompiles.extend([
            secp256k1::ECRECOVER,
            hash::SHA256,
            hash::RIPEMD160,
            identity::FUN,
            // EIP-196: Precompiled contracts for addition and scalar multiplication on the elliptic curve alt_bn128.
            // EIP-197: Precompiled contracts for optimal ate pairing check on the elliptic curve alt_bn128.
            bn128::add::BYZANTIUM,
            bn128::mul::BYZANTIUM,
            bn128::pair::BYZANTIUM,
            // EIP-198: Big integer modular exponentiation.
            modexp::BYZANTIUM,
        ]);
        precompiles
    }

    /// Returns precompiles for Istanbul spec.
    pub fn istanbul() -> Self {
        let mut precompiles = Precompiles::default();
        precompiles.extend([
            secp256k1::ECRECOVER,
            hash::SHA256,
            hash::RIPEMD160,
            identity::FUN,
            // EIP-152: Add BLAKE2 compression function `F` precompile.
            blake2::FUN,
            // EIP-1108: Reduce alt_bn128 precompile gas costs.
            bn128::add::ISTANBUL,
            bn128::mul::ISTANBUL,
            bn128::pair::ISTANBUL,
            modexp::BYZANTIUM,
        ]);
        precompiles
    }

    /// Returns precompiles for Berlin spec.
    pub fn berlin() -> Self {
        let mut precompiles = Self::default();
        precompiles.extend([
            secp256k1::ECRECOVER,
            hash::SHA256,
            hash::RIPEMD160,
            identity::FUN,
            blake2::FUN,
            bn128::add::ISTANBUL,
            bn128::mul::ISTANBUL,
            bn128::pair::ISTANBUL,
            // EIP-2565: ModExp Gas Cost.
            modexp::BERLIN,
        ]);
        precompiles
    }

    /// Returns precompiles for Cancun spec.
    ///
    /// If the `c-kzg` feature is not enabled KZG Point Evaluation precompile will not be included,
    /// effectively making this the same as Berlin.
    pub fn cancun() -> Self {
        let precompiles = Self::default();

        // Don't include KZG point evaluation precompile in no_std builds.
        #[cfg(feature = "c-kzg")]
        let precompiles = {
            let mut precompiles = precompiles;
            precompiles.extend([
                secp256k1::ECRECOVER,
                hash::SHA256,
                hash::RIPEMD160,
                identity::FUN,
                blake2::FUN,
                bn128::add::ISTANBUL,
                bn128::mul::ISTANBUL,
                bn128::pair::ISTANBUL,
                modexp::BERLIN,
            ]);
            precompiles.extend([
                // EIP-4844: Shard Blob Transactions
                kzg_point_evaluation::POINT_EVALUATION,
            ]);
            precompiles
        };

        precompiles
    }

    /// Returns the precompiles for the latest spec.
    pub fn latest() -> Self {
        Self::cancun()
    }

    /// Returns an iterator over the precompiles addresses.
    #[inline]
    pub fn addresses(&self) -> impl Iterator<Item = &Address> + '_ {
        self.inner.keys()
    }

    /// Consumes the type and returns all precompile addresses.
    #[inline]
    pub fn into_addresses(self) -> impl Iterator<Item = Address> {
        self.inner.into_keys()
    }

    /// Is the given address a precompile.
    #[inline]
    pub fn contains(&self, address: &Address) -> bool {
        self.inner.contains_key(address)
    }

    /// Returns the precompile for the given address.
    #[inline]
    pub fn get(&self, address: &Address) -> Option<&Precompile<CTX, EXTCXT>> {
        self.inner.get(address)
    }

    /// Returns the precompile for the given address.
    #[inline]
    pub fn get_mut(&mut self, address: &Address) -> Option<&mut Precompile<CTX, EXTCXT>> {
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

    /// Extends the precompiles with the given precompiles.
    ///
    /// Other precompiles with overwrite existing precompiles.
    pub fn extend(
        &mut self,
        other: impl IntoIterator<Item = impl Into<(Address, Precompile<CTX, EXTCXT>)>>,
    ) {
        self.inner.extend(other.into_iter().map(Into::into));
    }
}

#[derive(Clone, Debug)]
pub struct PrecompileWithAddress(pub Address, pub StandardPrecompileFn);

#[derive(Clone, Debug)]
pub struct EnvPrecompileWithAddress(pub Address, pub EnvPrecompileFn);

impl From<(Address, StandardPrecompileFn)> for PrecompileWithAddress {
    fn from(value: (Address, StandardPrecompileFn)) -> Self {
        PrecompileWithAddress(value.0, value.1)
    }
}

impl From<PrecompileWithAddress> for (Address, StandardPrecompileFn) {
    fn from(value: PrecompileWithAddress) -> Self {
        (value.0, value.1)
    }
}

impl<CTX, EXTCXT> From<PrecompileWithAddress> for (Address, Precompile<CTX, EXTCXT>) {
    fn from(value: PrecompileWithAddress) -> Self {
        (value.0, Precompile::Standard(value.1))
    }
}

impl<CTX, EXTCXT> From<EnvPrecompileWithAddress> for (Address, Precompile<CTX, EXTCXT>) {
    fn from(value: EnvPrecompileWithAddress) -> Self {
        (value.0, Precompile::Env(value.1))
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum PrecompileSpecId {
    HOMESTEAD,
    BYZANTIUM,
    ISTANBUL,
    BERLIN,
    CANCUN,
    LATEST,
}

impl PrecompileSpecId {
    /// Returns the appropriate precompile Spec for the primitive [SpecId](revm_primitives::SpecId)
    pub const fn from_spec_id(spec_id: revm_primitives::SpecId) -> Self {
        use revm_primitives::SpecId::*;
        match spec_id {
            FRONTIER | FRONTIER_THAWING | HOMESTEAD | DAO_FORK | TANGERINE | SPURIOUS_DRAGON => {
                Self::HOMESTEAD
            }
            BYZANTIUM | CONSTANTINOPLE | PETERSBURG => Self::BYZANTIUM,
            ISTANBUL | MUIR_GLACIER => Self::ISTANBUL,
            BERLIN | LONDON | ARROW_GLACIER | GRAY_GLACIER | MERGE | SHANGHAI => Self::BERLIN,
            CANCUN => Self::CANCUN,
            LATEST => Self::LATEST,
            #[cfg(feature = "optimism")]
            BEDROCK | REGOLITH | CANYON => Self::BERLIN,
            #[cfg(feature = "optimism")]
            ECOTONE => Self::CANCUN,
        }
    }
}

/// Const function for making an address by concatenating the bytes from two given numbers.
///
/// Note that 32 + 128 = 160 = 20 bytes (the length of an address). This function is used
/// as a convenience for specifying the addresses of the various precompiles.
#[inline]
const fn u64_to_address(x: u64) -> Address {
    let x = x.to_be_bytes();
    Address::new([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, x[0], x[1], x[2], x[3], x[4], x[5], x[6], x[7],
    ])
}
