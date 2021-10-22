use crate::{
    collection::Vec,
    Spec, SpecId,
};
//pub(crate) use crate::precompiles::secp256k1::ecrecover;
use crate::{
    models::CallContext,
    precompiles::{
        blake2::Blake2F,
        bn128::{Bn128Add, Bn128Mul, Bn128Pair},
        hash::{RIPEMD160, SHA256},
        identity::Identity,
        modexp::ModExp,
        secp256k1::ECRecover,
    },
    ExitError, Log,
};
use primitive_types::{H160 as Address, U256};

mod blake2;
mod bn128;
mod hash;
mod identity;
mod modexp;
mod secp256k1;

pub fn calc_linear_cost_u32(len: usize, base: u64, word: u64) -> u64 {
    (len as u64 + 32 - 1) / 32 * word + base
}

pub fn gas_quert(gas_used: u64, gas_limit: u64) -> Result<u64, ExitError> {
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

type EvmPrecompileResult = Result<PrecompileOutput, ExitError>;

/// A precompiled function for use in the EVM.
pub trait Precompile {
    /// Runs the precompile function.
    fn run(
        input: &[u8],
        target_gas: u64,
        machine: &CallContext,
        is_static: bool,
    ) -> PrecompileResult;
}

/// Hard fork marker.
pub trait HardFork {}

/// Homestead hard fork marker.
pub struct Homestead;

/// Homestead hard fork marker.
pub struct Byzantium;

/// Homestead hard fork marker.
pub struct Istanbul;

/// Homestead hard fork marker.
pub struct Berlin;

impl HardFork for Homestead {}

impl HardFork for Byzantium {}

impl HardFork for Istanbul {}

impl HardFork for Berlin {}

pub type PrecompileFn = fn(&[u8], u64, &CallContext, bool) -> PrecompileResult;

pub struct Precompiles {
    addresses: Vec<Address>,
    fun: Vec<PrecompileFn>,
}

impl Precompiles {
    //TODO refactor this
    pub fn new<SPEC: Spec>() -> Self {
        let mut add = Vec::new();
        let mut fun: Vec<PrecompileFn> = Vec::new();
        if SPEC::enabled(SpecId::HOMESTEAD) {
            add.push(ECRecover::ADDRESS);
            add.push(SHA256::ADDRESS);
            add.push(RIPEMD160::ADDRESS);

            fun.push(ECRecover::run);
            fun.push(SHA256::run);
            fun.push(RIPEMD160::run);
        }
        if SPEC::enabled(SpecId::BYZANTINE) {
            add.push(Identity::ADDRESS);
            fun.push(Identity::run);
        }

        if SPEC::enabled(SpecId::ISTANBUL) {
            // EIP-152: Add BLAKE2 compression function `F` precompile
            add.push(Blake2F::ADDRESS);
            fun.push(Blake2F::run);
        }

        if SPEC::enabled(SpecId::ISTANBUL) {
            // EIP-1108: Reduce alt_bn128 precompile gas costs
            add.push(Bn128Add::<Istanbul>::ADDRESS);
            add.push(Bn128Mul::<Istanbul>::ADDRESS);
            add.push(Bn128Pair::<Istanbul>::ADDRESS);

            fun.push(Bn128Add::<Istanbul>::run);
            fun.push(Bn128Mul::<Istanbul>::run);
            fun.push(Bn128Pair::<Istanbul>::run);
        } else if SPEC::enabled(SpecId::BYZANTINE) {
            // EIP-196: Precompiled contracts for addition and scalar multiplication on the elliptic curve alt_bn128
            // EIP-197: Precompiled contracts for optimal ate pairing check on the elliptic curve alt_bn128
            add.push(Bn128Add::<Byzantium>::ADDRESS);
            add.push(Bn128Mul::<Byzantium>::ADDRESS);
            add.push(Bn128Pair::<Byzantium>::ADDRESS);

            fun.push(Bn128Add::<Byzantium>::run);
            fun.push(Bn128Mul::<Byzantium>::run);
            fun.push(Bn128Pair::<Byzantium>::run);
        }

        if SPEC::enabled(SpecId::BERLIN) {
            add.push(ModExp::<Berlin>::ADDRESS);
            fun.push(ModExp::<Berlin>::run);
        } else if SPEC::enabled(SpecId::BYZANTINE) {
            //EIP-198: Big integer modular exponentiation
            add.push(ModExp::<Byzantium>::ADDRESS);
            fun.push(ModExp::<Byzantium>::run);
        }

        Self {
            addresses: add,
            fun,
        }
    }

    pub fn addresses(&self) -> &[Address] {
        &self.addresses
    }

    pub fn get_fun(&self, address: &Address) -> Option<PrecompileFn> {
        //return None;
        if let Some(index) = self.addresses.iter().position(|t| t == address) {
            self.fun.get(index).cloned()
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
