#![no_std]

use bytes::Bytes;
use primitive_types::{H160 as Address, H256, U256};

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

pub struct Precompiles {
    fun: Vec<(Address, Precompile)>,
}

impl Default for Precompiles {
    fn default() -> Self {
        Self::new::<3>() //berlin
    }
}

#[derive(Clone)]
pub enum Precompile {
    Standard(StandardPrecompileFn),
    Custom(CustomPrecompileFn),
}

pub enum SpecId {
    HOMESTEAD = 0,
    BYZANTIUM = 1,
    ISTANBUL = 2,
    BERLIN = 3,
}

impl SpecId {
    pub const fn enabled(self, spec_id: u8) -> bool {
        spec_id as u8 >= self as u8
    }
}

impl Precompiles {
    pub fn new<const SPEC_ID: u8>() -> Self {
        let mut fun: Vec<(Address, Precompile)> = Vec::new();
        if SpecId::HOMESTEAD.enabled(SPEC_ID) {
            fun.push(secp256k1::ECRECOVER);
            fun.push(hash::SHA256);
            fun.push(hash::RIPED160);
            fun.push(identity::FUN);
        }

        if SpecId::ISTANBUL.enabled(SPEC_ID) {
            // EIP-152: Add BLAKE2 compression function `F` precompile
            fun.push(blake2::FUN);
        }

        if SpecId::ISTANBUL.enabled(SPEC_ID) {
            // EIP-1108: Reduce alt_bn128 precompile gas costs
            fun.push(bn128::add::ISTANBUL);
            fun.push(bn128::mul::ISTANBUL);
            fun.push(bn128::pair::ISTANBUL);
        } else if SpecId::BYZANTIUM.enabled(SPEC_ID) {
            // EIP-196: Precompiled contracts for addition and scalar multiplication on the elliptic curve alt_bn128
            // EIP-197: Precompiled contracts for optimal ate pairing check on the elliptic curve alt_bn128
            fun.push(bn128::add::BYZANTIUM);
            fun.push(bn128::mul::BYZANTIUM);
            fun.push(bn128::pair::BYZANTIUM);
        }

        if SpecId::BERLIN.enabled(SPEC_ID) {
            fun.push(modexp::BERLIN);
        } else if SpecId::BYZANTIUM.enabled(SPEC_ID) {
            //EIP-198: Big integer modular exponentiation
            fun.push(modexp::BYZANTIUM);
        }

        Self { fun }
    }

    pub fn as_slice(&self) -> &[(Address, Precompile)] {
        &self.fun
    }

    pub fn contains(&self, address: &Address) -> bool {
        matches!(self.get(address), Some(_))
    }

    pub fn get(&self, address: &Address) -> Option<Precompile> {
        //return None;
        self.fun
            .iter()
            .find(|(t, _)| t == address)
            .map(|(_, precompile)| precompile.clone())
    }
    
    pub fn len(&self) -> usize {
        self.fun.len()
    }
}

/// const fn for making an address by concatenating the bytes from two given numbers,
/// Note that 32 + 128 = 160 = 20 bytes (the length of an address). This function is used
/// as a convenience for specifying the addresses of the various precompiles.
const fn make_address(x: u32, y: u128) -> Address {
    let x_bytes = x.to_be_bytes();
    let y_bytes = y.to_be_bytes();
    Address([
        x_bytes[0],
        x_bytes[1],
        x_bytes[2],
        x_bytes[3],
        y_bytes[0],
        y_bytes[1],
        y_bytes[2],
        y_bytes[3],
        y_bytes[4],
        y_bytes[5],
        y_bytes[6],
        y_bytes[7],
        y_bytes[8],
        y_bytes[9],
        y_bytes[10],
        y_bytes[11],
        y_bytes[12],
        y_bytes[13],
        y_bytes[14],
        y_bytes[15],
    ])
}

//use for test
pub fn u256_to_arr(value: &U256) -> [u8; 32] {
    let mut result = [0u8; 32];
    value.to_big_endian(&mut result);
    result
}
