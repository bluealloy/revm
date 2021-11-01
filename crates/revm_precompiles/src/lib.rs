#![no_std]

use bytes::Bytes;
use primitive_types::{H160 as Address, H256, U256};

mod blake2;
mod bn128;
mod error;
mod hash;
mod identity;
mod modexp;

#[cfg(feature = "secp256k1")]
mod secp256k1;

pub use error::ExitError;

/// libraries for no_std flag
#[macro_use]
extern crate alloc;
use alloc::vec::Vec;

pub fn calc_linear_cost_u32(len: usize, base: u64, word: u64) -> u64 {
    (len as u64 + 32 - 1) / 32 * word + base
}

pub fn gas_query(gas_used: u64, gas_limit: u64) -> Result<u64, ExitError> {
    if gas_used > gas_limit {
        return Err(ExitError::OutOfGas);
    }
    Ok(gas_used)
}

#[derive(Debug)]
pub struct PrecompileOutput {
    pub cost: u64,
    pub output: Vec<u8>,
    pub logs: Vec<Log>,
}

#[derive(Debug)]
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

impl Default for PrecompileOutput {
    fn default() -> Self {
        PrecompileOutput {
            cost: 0,
            output: Vec::new(),
            logs: Vec::new(),
        }
    }
}

/// A precompile operation result.
pub type PrecompileResult = Result<PrecompileOutput, ExitError>;

pub type StandardPrecompileFn = fn(&[u8], u64) -> PrecompileResult;
pub type CustomPrecompileFn = fn(&[u8], u64) -> PrecompileResult;

pub struct Precompiles {
    fun: Vec<(Address, Precompile)>,
}

#[derive(Clone)]
pub enum Precompile {
    Standard(StandardPrecompileFn),
    Custom(CustomPrecompileFn),
}

pub enum SpecId {
    HOMESTEAD = 0,
    BYZANTINE = 1,
    ISTANBUL = 2,
    BERLIN = 3,
}

impl SpecId {
    pub const fn enabled(self, spec_id: u8) -> bool {
        spec_id as u8 >= self as u8
    }
}

impl Precompiles {
    //TODO refactor this
    pub fn new<const SPEC_ID: u8>() -> Self {
        let mut fun: Vec<(Address, Precompile)> = Vec::new();
        if SpecId::HOMESTEAD.enabled(SPEC_ID) {
            fun.push(hash::SHA256);
            fun.push(hash::RIPED160);
            #[cfg(feature = "secp256k1")]
            fun.push(secp256k1::ECRECOVER);
            // TODO check if this goes here
            fun.push(identity::FUN);
        }
        if SpecId::BYZANTINE.enabled(SPEC_ID) {}

        if SpecId::ISTANBUL.enabled(SPEC_ID) {
            // EIP-152: Add BLAKE2 compression function `F` precompile
            fun.push(blake2::FUN);
        }

        if SpecId::ISTANBUL.enabled(SPEC_ID) {
            // EIP-1108: Reduce alt_bn128 precompile gas costs
            fun.push(bn128::add::ISTANBUL);
            fun.push(bn128::mul::ISTANBUL);
            fun.push(bn128::pair::ISTANBUL);
        } else if SpecId::BYZANTINE.enabled(SPEC_ID) {
            // EIP-196: Precompiled contracts for addition and scalar multiplication on the elliptic curve alt_bn128
            // EIP-197: Precompiled contracts for optimal ate pairing check on the elliptic curve alt_bn128
            fun.push(bn128::add::BYZANTIUM);
            fun.push(bn128::mul::BYZANTIUM);
            fun.push(bn128::pair::BYZANTIUM);
        }

        if SpecId::BERLIN.enabled(SPEC_ID) {
            fun.push(modexp::BERLIN);
        } else if SpecId::BYZANTINE.enabled(SPEC_ID) {
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
        if let Some((_, precompile)) = self.fun.iter().find(|(t, _)| t == address) {
            Some(precompile.clone())
        } else {
            None
        }
    }
}

/// Matches the address given to Homestead precompiles.
// impl<'backend, 'config> executor::Precompiles<AuroraStackState<'backend, 'config>> for Precompiles {
//     fn run(
//         &self,
//         address: Address,
//         input: &[u8],
//         target_gas: Option<u64>,
//         context: &CallContext,
//         state: &mut AuroraStackState,
//         is_static: bool,
//     ) -> Option<EvmPrecompileResult> {
//         let target_gas = match target_gas {
//             Some(t) => t,
//             None => return Some(EvmPrecompileResult::Err(ExitError::OutOfGas)),
//         };

//         let output = self.get_fun(&address).map(|fun| {
//             let mut res = (fun)(input, target_gas, context, is_static);
//             if let Ok(output) = &mut res {
//                 if let Some(promise) = output.promise.take() {
//                     state.add_promise(promise)
//                 }
//             }
//             res
//         });

//         output.map(|res| res.map(Into::into))
//     }

//     fn addresses(&self) -> &[Address] {
//         &self.addresses
//     }
// }

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

/*
#[cfg(test)]
mod tests {
    use crate::precompiles::{Byzantium, Istanbul};
    use crate::prelude::Address;
    use rand::Rng;

    #[test]
    fn test_precompile_addresses() {
        assert_eq!(super::secp256k1::ECRecover::ADDRESS, u8_to_address(1));
        assert_eq!(super::hash::SHA256::ADDRESS, u8_to_address(2));
        assert_eq!(super::hash::RIPEMD160::ADDRESS, u8_to_address(3));
        assert_eq!(super::identity::Identity::ADDRESS, u8_to_address(4));
        assert_eq!(super::ModExp::<Byzantium>::ADDRESS, u8_to_address(5));
        assert_eq!(super::Bn128Add::<Istanbul>::ADDRESS, u8_to_address(6));
        assert_eq!(super::Bn128Mul::<Istanbul>::ADDRESS, u8_to_address(7));
        assert_eq!(super::Bn128Pair::<Istanbul>::ADDRESS, u8_to_address(8));
        assert_eq!(super::blake2::Blake2F::ADDRESS, u8_to_address(9));
    }

    #[test]
    fn test_make_address() {
        for i in 0..u8::MAX {
            assert_eq!(super::make_address(0, i as u128), u8_to_address(i));
        }

        let mut rng = rand::thread_rng();
        for _ in 0..u8::MAX {
            let address: Address = Address(rng.gen());
            let (x, y) = split_address(address);
            assert_eq!(address, super::make_address(x, y))
        }
    }

    fn u8_to_address(x: u8) -> Address {
        let mut bytes = [0u8; 20];
        bytes[19] = x;
        Address(bytes)
    }

    // Inverse function of `super::make_address`.
    fn split_address(a: Address) -> (u32, u128) {
        let mut x_bytes = [0u8; 4];
        let mut y_bytes = [0u8; 16];

        x_bytes.copy_from_slice(&a[0..4]);
        y_bytes.copy_from_slice(&a[4..20]);

        (u32::from_be_bytes(x_bytes), u128::from_be_bytes(y_bytes))
    }
}
*/
