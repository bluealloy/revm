//! # revm-precompile
//!
//! Implementations of EVM precompiled contracts.
#![warn(unused_crate_dependencies)]
#![deny(unused_must_use, rust_2018_idioms)]
#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

#[macro_use]
extern crate alloc;

mod blake2;
mod bn128;
mod hash;
mod identity;
#[cfg(feature = "c-kzg")]
pub mod kzg_point_evaluation;
mod modexp;
mod secp256k1;
pub mod utilities;

use alloc::{boxed::Box, collections::BTreeMap, vec::Vec};
use core::{fmt, hash::Hash};
use once_cell::race::OnceBox;
#[doc(hidden)]
pub use revm_primitives as primitives;
pub use revm_primitives::{
    precompile::{PrecompileError as Error, *},
    Address, Bytes, HashMap, B256,
};

pub fn calc_linear_cost_u32(len: usize, base: u64, word: u64) -> u64 {
    (len as u64 + 32 - 1) / 32 * word + base
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct PrecompileOutput {
    pub cost: u64,
    pub output: Vec<u8>,
    pub logs: Vec<Log>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Log {
    pub address: Address,
    pub topics: Vec<B256>,
    pub data: Bytes,
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
#[derive(Clone, Debug)]
pub struct Precompiles {
    pub inner: Vec<PrecompileWithAddress>,
}

impl Precompiles {
    /// Returns the precompiles for the given spec.
    pub fn new(spec: SpecId) -> &'static Self {
        match spec {
            SpecId::HOMESTEAD => Self::homestead(),
            SpecId::BYZANTIUM => Self::byzantium(),
            SpecId::ISTANBUL => Self::istanbul(),
            SpecId::BERLIN => Self::berlin(),
            SpecId::CANCUN => Self::cancun(),
            SpecId::LATEST => Self::latest(),
        }
    }

    /// Returns precompiles for Homestead spec.
    pub fn homestead() -> &'static Self {
        static INSTANCE: OnceBox<Precompiles> = OnceBox::new();
        INSTANCE.get_or_init(|| {
            let mut inner = vec![
                secp256k1::ECRECOVER,
                hash::SHA256,
                hash::RIPEMD160,
                identity::FUN,
            ];
            inner.sort_unstable_by_key(|i| i.0);
            Box::new(Self { inner })
        })
    }

    /// Returns precompiles for Byzantium spec.
    pub fn byzantium() -> &'static Self {
        static INSTANCE: OnceBox<Precompiles> = OnceBox::new();
        INSTANCE.get_or_init(|| {
            let mut precompiles = Self::homestead().clone();
            precompiles.extend([
                // EIP-196: Precompiled contracts for addition and scalar multiplication on the elliptic curve alt_bn128.
                // EIP-197: Precompiled contracts for optimal ate pairing check on the elliptic curve alt_bn128.
                bn128::add::BYZANTIUM,
                bn128::mul::BYZANTIUM,
                bn128::pair::BYZANTIUM,
                // EIP-198: Big integer modular exponentiation.
                modexp::BYZANTIUM,
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
                // EIP-152: Add BLAKE2 compression function `F` precompile.
                blake2::FUN,
                // EIP-1108: Reduce alt_bn128 precompile gas costs.
                bn128::add::ISTANBUL,
                bn128::mul::ISTANBUL,
                bn128::pair::ISTANBUL,
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
    /// If `std` feature is not enabled KZG Point Evaluation precompile will not be included.
    pub fn cancun() -> &'static Self {
        static INSTANCE: OnceBox<Precompiles> = OnceBox::new();
        INSTANCE.get_or_init(|| {
            let precompiles = Self::berlin().clone();

            // Don't include KZG point evaluation precompile in no_std builds.
            #[cfg(feature = "c-kzg")]
            let precompiles = {
                let mut precompiles = precompiles;
                precompiles.extend([
                    // EIP-4844: Shard Blob Transactions
                    kzg_point_evaluation::POINT_EVALUATION,
                ]);
                precompiles
            };

            Box::new(precompiles)
        })
    }

    /// Returns the precompiles for the latest spec.
    pub fn latest() -> &'static Self {
        Self::cancun()
    }

    /// Returns an iterator over the precompiles addresses.
    #[inline]
    pub fn addresses(&self) -> impl Iterator<Item = &Address> + '_ {
        self.inner.iter().map(|i| &i.0)
    }

    /// Consumes the type and returns all precompile addresses.
    #[inline]
    pub fn into_addresses(self) -> impl Iterator<Item = Address> {
        self.inner.into_iter().map(|precompile| precompile.0)
    }

    /// Is the given address a precompile.
    #[inline]
    pub fn contains(&self, address: &Address) -> bool {
        self.get(address).is_some()
    }

    /// Returns the precompile for the given address.
    #[inline]
    pub fn get(&self, address: &Address) -> Option<Precompile> {
        //return None;
        self.inner
            .binary_search_by_key(address, |i| i.0)
            .ok()
            .map(|i| self.inner[i].1.clone())
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
    pub fn extend(&mut self, other: impl IntoIterator<Item = PrecompileWithAddress>) {
        self.inner = self
            .inner
            .iter()
            .cloned()
            .chain(other)
            .map(|i| (i.0, i.1.clone()))
            .collect::<BTreeMap<Address, Precompile>>()
            .into_iter()
            .map(|(k, v)| PrecompileWithAddress(k, v))
            .collect::<Vec<_>>();
    }
}

impl Default for Precompiles {
    fn default() -> Self {
        Self::new(SpecId::LATEST).clone() //berlin
    }
}

#[derive(Clone)]
pub enum Precompile {
    Standard(StandardPrecompileFn),
    Env(EnvPrecompileFn),
}

impl fmt::Debug for Precompile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Precompile::Standard(_) => f.write_str("Standard"),
            Precompile::Env(_) => f.write_str("Env"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PrecompileWithAddress(Address, Precompile);

impl From<PrecompileWithAddress> for (Address, Precompile) {
    fn from(value: PrecompileWithAddress) -> Self {
        (value.0, value.1)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum SpecId {
    HOMESTEAD,
    BYZANTIUM,
    ISTANBUL,
    BERLIN,
    CANCUN,
    LATEST,
}

impl SpecId {
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
            BEDROCK | REGOLITH => Self::BERLIN,
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
