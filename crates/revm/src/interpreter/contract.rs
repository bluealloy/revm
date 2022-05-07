use crate::{alloc::vec::Vec, opcode::spec_opcode_gas, CallContext, Spec};
use bytes::Bytes;
use primitive_types::{H160, U256};

use crate::instructions::opcode;

pub struct Contract {
    /// Contracts data
    pub input: Bytes,
    /// Contract code
    pub code: Bytes,
    /// code size of original code. Note that current code is extended with push padding and STOP at end
    pub code_size: usize,
    /// Contract address
    pub address: H160,
    /// Caller of the EVM.
    pub caller: H160,
    /// Value send to contract.
    pub value: U256,
    /// Precomputed valid jump addresses
    jumpdest: ValidJumpAddress,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Analysis {
    JumpDest,
    GasBlockEnd, //contains gas for next block
    None,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AnalysisData {
    pub is_jumpdest: bool,
    pub gas_block: u64,
}

impl AnalysisData {
    pub fn none() -> Self {
        AnalysisData {
            is_jumpdest: false,
            gas_block: 0,
        }
    }

    pub fn jump_dest() -> Self {
        AnalysisData {
            is_jumpdest: true,
            gas_block: 0,
        }
    }

    pub fn is_jump_dest(&self) -> bool {
        self.is_jumpdest
    }
}

impl Contract {
    pub fn new<SPEC: Spec>(
        input: Bytes,
        code: Bytes,
        address: H160,
        caller: H160,
        value: U256,
    ) -> Self {
        let code_size = code.len();
        let (jumpdest, code) = Self::analyze::<SPEC>(code.as_ref());

        let code = code.into();
        Self {
            input,
            code,
            code_size,
            address,
            caller,
            value,
            jumpdest,
        }
    }

    /// Create a new valid mapping from given code bytes.
    /// it gives back ValidJumpAddress and size od needed paddings.
    fn analyze<SPEC: Spec>(code: &[u8]) -> (ValidJumpAddress, Vec<u8>) {
        let mut jumps: Vec<AnalysisData> = Vec::with_capacity(code.len() + 33);
        // padding of PUSH32 size plus one for stop
        jumps.resize(code.len() + 33, AnalysisData::none());
        //let opcode_gas = LONDON_OPCODES;
        let opcode_gas = spec_opcode_gas(SPEC::SPEC_ID);
        let mut index = 0;
        let mut first_gas_block: u64 = 0;
        let mut block_start: usize = 0;
        // first gas block

        while index < code.len() {
            let opcode = unsafe { *code.get_unchecked(index) };
            let info = unsafe { opcode_gas.get_unchecked(opcode as usize) };
            first_gas_block += info.gas;

            index += if info.is_push {
                ((opcode - opcode::PUSH1) + 2) as usize
            } else {
                1
            };

            if info.is_gas_block_end {
                block_start = index - 1;
                if info.is_jump {
                    unsafe {
                        jumps.get_unchecked_mut(block_start).is_jumpdest = true;
                    }
                }
                break;
            }
        }

        let mut gas_in_block: u64 = 0;
        while index < code.len() {
            let opcode = unsafe { *code.get_unchecked(index) };
            let info = unsafe { opcode_gas.get_unchecked(opcode as usize) };
            gas_in_block += info.gas;

            if info.is_gas_block_end {
                if info.is_jump {
                    unsafe {
                        jumps.get_unchecked_mut(index).is_jumpdest = true;
                    }
                }
                unsafe {
                    jumps.get_unchecked_mut(block_start).gas_block = gas_in_block;
                }
                block_start = index;
                gas_in_block = 0;
            }

            index += if info.is_push {
                ((opcode - opcode::PUSH1) + 2) as usize
            } else {
                1
            };
        }
        if gas_in_block != 0 {
            unsafe {
                jumps.get_unchecked_mut(block_start).gas_block = gas_in_block;
            }
        }
        let padding = index - code.len();
        // +1 is for forced STOP opcode at the end of contract, it is precausion
        // if there is none, and if there is STOP our additional opcode will do nothing.
        //jumps.resize(jumps.len() + padding + 1, AnalysisData::none());
        let mut code = code.to_vec();
        code.resize(code.len() + padding + 1, 0);

        (ValidJumpAddress::new(jumps, first_gas_block), code)
    }

    pub fn is_valid_jump(&self, possition: usize) -> bool {
        self.jumpdest.is_valid(possition)
    }

    pub fn gas_block(&self, possition: usize) -> u64 {
        self.jumpdest.gas_block(possition)
    }
    pub fn first_gas_block(&self) -> u64 {
        self.jumpdest.first_gas_block
    }

    pub fn new_with_context<SPEC: Spec>(
        input: Bytes,
        code: Bytes,
        call_context: &CallContext,
    ) -> Self {
        Self::new::<SPEC>(
            input,
            code,
            call_context.address,
            call_context.caller,
            call_context.apparent_value,
        )
    }
}

/// Mapping of valid jump destination from code.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidJumpAddress {
    first_gas_block: u64,
    analysis: Vec<AnalysisData>,
}

impl ValidJumpAddress {
    pub fn new(analysis: Vec<AnalysisData>, first_gas_block: u64) -> Self {
        Self {
            analysis,
            first_gas_block,
        }
    }
    /// Get the length of the valid mapping. This is the same as the
    /// code bytes.

    pub fn len(&self) -> usize {
        self.analysis.len()
    }

    /// Returns true if the valid list is empty

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns `true` if the position is a valid jump destination. If
    /// not, returns `false`.

    pub fn is_valid(&self, position: usize) -> bool {
        if position >= self.analysis.len() {
            return false;
        }

        self.analysis[position].is_jump_dest()
    }

    pub fn gas_block(&self, position: usize) -> u64 {
        self.analysis[position].gas_block
    }
}

/*
#[cfg(test)]
mod tests {

    use std::time::Instant;

    use super::*;
    use crate::LondonSpec;

    #[test]
    pub fn analyze() {
        let bytes = hex::decode("3dcd25e7dac7413679ca4b860b371699db4a3c06cc8086e35490de6810b5510e792551b0049bc91b54aec6a82b00a85f998982b99dac982fb6f7acf67032fe9f23491f6c29a96be377f44ece4b89ab835bb2d36e387533e9b36e1c47b85c09175488dec63aca38f96d78cf8e468b54486b83c1d8db5931b5579a56bd1aa05d6526251556bfabac7c244c41e6a78f581aded59e297f6af196279d246b99a8670b5edddb646ed751417b70f1066f19dfea1c06e91e0beeb3a2511603d32092a0189f820ea97eb234a42ed8b513144971c4166e48b209d74b1d85e79f93094e901376e964bc2a8141f189f13edc69c97467a09b43c19140df1399a4740c6dfcced5b3d3d08abd97b3c71cfd1c2b95dd4b8ce7951bf9e17bdf35e0fd706e89551a1e7b79cfeedf3037eb1e99537da2c65f3acd7c3c1f47343f536566cd4976002870267f87d1b5066e158fb794185a0ec8a786bca89412bab10a167ba4e2087e37b7c7d4ab98f6c86abc59135bbb07d5c19f028724031be46e69fc1215fe5a8743f8ffb57294989cc3fa6dd9d38a2317ba6de811b9d135ea03b4ab5a2fd034454d2a0a59ec85deb5b05bcf3b6408e0a1d2d6a8b259510b49e2ea7479b9770f42fef2805a4a7cfea63714e0fd00929e293648ee5a57df894ab7cb46e331d120ba83c9e51ccab2c1ec8afe2809e0c3184c607e57045f95062abd78b1974192f542b12300000000000000").unwrap();
        let mut t = Vec::new();
        for _ in 0..40 {
            let time = Instant::now();
            for _ in 0..30_000 {
                let t = Contract::analyze::<LondonSpec>(&bytes);
            }
            t.push(time.elapsed());
        }
        t.sort();

        for i in t {
            println!("Elapsed: {:?}", i);
        }
    }
}
*/
