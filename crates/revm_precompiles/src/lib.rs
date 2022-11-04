#![no_std]

use bytes::Bytes;
use once_cell::sync::OnceCell;
use ruint::aliases::{B160 as Address, B256 as H256};

mod blake2;
mod bn128;
mod error;
mod hash;
mod identity;
mod modexp;
mod secp256k1;

pub use error::Return;

/// libraries for no_std flag
#[macro_use]
extern crate alloc;
use alloc::vec::Vec;
use core::fmt;

use hashbrown::HashMap;

pub fn calc_linear_cost_u32(len: usize, base: u64, word: u64) -> u64 {
    (len as u64 + 32 - 1) / 32 * word + base
}

pub fn gas_query(gas_used: u64, gas_limit: u64) -> Result<u64, Return> {
    if gas_used > gas_limit {
        return Err(Return::OutOfGas);
    }
    Ok(gas_used)
}

#[derive(Debug)]
pub struct PrecompileOutput {
    pub cost: u64,
    pub output: Vec<u8>,
    pub logs: Vec<Log>,
}

#[derive(Debug, Default)]
pub struct Log {
    pub address: Address,
    pub topics: Vec<H256>,
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

/// A precompile operation result.
pub type PrecompileResult = Result<PrecompileOutput, Return>;

pub type StandardPrecompileFn = fn(&[u8], u64) -> PrecompileResult;
pub type CustomPrecompileFn = fn(&[u8], u64) -> PrecompileResult;

#[derive(Clone, Debug)]
pub struct Precompiles {
    fun: HashMap<Address, Precompile>,
}

impl Default for Precompiles {
    fn default() -> Self {
        Self::new(SpecId::LATEST).clone() //berlin
    }
}

#[derive(Clone)]
pub enum Precompile {
    Standard(StandardPrecompileFn),
    Custom(CustomPrecompileFn),
}

impl fmt::Debug for Precompile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Precompile::Standard(_) => f.write_str("Standard"),
            Precompile::Custom(_) => f.write_str("Custom"),
        }
    }
}

pub struct PrecompileAddress(Address, Precompile);

impl From<PrecompileAddress> for (Address, Precompile) {
    fn from(value: PrecompileAddress) -> Self {
        (value.0, value.1)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum SpecId {
    HOMESTEAD = 0,
    BYZANTIUM = 1,
    ISTANBUL = 2,
    BERLIN = 3,
    LATEST = 4,
}

impl SpecId {
    pub const fn enabled(self, spec_id: u8) -> bool {
        spec_id >= self as u8
    }
}

impl Precompiles {
    pub fn homestead() -> &'static Self {
        static INSTANCE: OnceCell<Precompiles> = OnceCell::new();
        INSTANCE.get_or_init(|| {
            let fun = vec![
                secp256k1::ECRECOVER,
                hash::SHA256,
                hash::RIPEMD160,
                identity::FUN,
            ]
            .into_iter()
            .map(From::from)
            .collect();
            Self { fun }
        })
    }

    pub fn byzantium() -> &'static Self {
        static INSTANCE: OnceCell<Precompiles> = OnceCell::new();
        INSTANCE.get_or_init(|| {
            let mut precompiles = Self::homestead().clone();
            precompiles.fun.extend(
                vec![
                    // EIP-196: Precompiled contracts for addition and scalar multiplication on the elliptic curve alt_bn128.
                    // EIP-197: Precompiled contracts for optimal ate pairing check on the elliptic curve alt_bn128.
                    bn128::add::BYZANTIUM,
                    bn128::mul::BYZANTIUM,
                    bn128::pair::BYZANTIUM,
                    // EIP-198: Big integer modular exponentiation.
                    modexp::BYZANTIUM,
                ]
                .into_iter()
                .map(From::from),
            );
            precompiles
        })
    }

    pub fn istanbul() -> &'static Self {
        static INSTANCE: OnceCell<Precompiles> = OnceCell::new();
        INSTANCE.get_or_init(|| {
            let mut precompiles = Self::byzantium().clone();
            precompiles.fun.extend(
                vec![
                    // EIP-152: Add BLAKE2 compression function `F` precompile.
                    blake2::FUN,
                    // EIP-1108: Reduce alt_bn128 precompile gas costs.
                    bn128::add::ISTANBUL,
                    bn128::mul::ISTANBUL,
                    bn128::pair::ISTANBUL,
                ]
                .into_iter()
                .map(From::from),
            );
            precompiles
        })
    }

    pub fn berlin() -> &'static Self {
        static INSTANCE: OnceCell<Precompiles> = OnceCell::new();
        INSTANCE.get_or_init(|| {
            let mut precompiles = Self::istanbul().clone();
            precompiles.fun.extend(
                vec![
                    // EIP-2565: ModExp Gas Cost.
                    modexp::BERLIN,
                ]
                .into_iter()
                .map(From::from),
            );
            precompiles
        })
    }

    pub fn latest() -> &'static Self {
        Self::berlin()
    }

    pub fn new(spec: SpecId) -> &'static Self {
        match spec {
            SpecId::HOMESTEAD => Self::homestead(),
            SpecId::BYZANTIUM => Self::byzantium(),
            SpecId::ISTANBUL => Self::istanbul(),
            SpecId::BERLIN => Self::berlin(),
            SpecId::LATEST => Self::latest(),
        }
    }

    pub fn addresses(&self) -> impl IntoIterator<Item = &Address> {
        self.fun.keys()
    }

    pub fn contains(&self, address: &Address) -> bool {
        self.fun.contains_key(address)
    }

    pub fn get(&self, address: &Address) -> Option<Precompile> {
        //return None;
        self.fun.get(address).cloned()
    }

    pub fn is_empty(&self) -> bool {
        self.fun.len() == 0
    }

    pub fn len(&self) -> usize {
        self.fun.len()
    }
}
