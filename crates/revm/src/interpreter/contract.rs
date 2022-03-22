use crate::{
    alloc::vec::Vec,
    opcode::{spec_opcode_gas, OpType},
    CallContext, Spec,
};
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
        let mut jumps: Vec<AnalysisData> = Vec::with_capacity(code.len());
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

            match info.optype {
                OpType::Ordinary => {
                    index += 1;
                }
                OpType::GasBlockEnd => {
                    block_start = index;
                    index += 1;
                    break;
                }
                OpType::Push => {
                    index += ((opcode - opcode::PUSH1) + 2) as usize;
                }
                OpType::JumpDest => {
                    unsafe {
                        jumps.get_unchecked_mut(index).is_jumpdest = true;
                    }
                    block_start = index;
                    index += 1;
                    break;
                }
            }
        }

        let mut gas_in_block: u64 = 0;
        while index < code.len() {
            let opcode = unsafe { *code.get_unchecked(index) };
            let info = unsafe { opcode_gas.get_unchecked(opcode as usize) };
            gas_in_block += info.gas;

            match info.optype {
                OpType::Ordinary => {
                    index += 1;
                }
                OpType::GasBlockEnd => {
                    unsafe {
                        jumps.get_unchecked_mut(block_start).gas_block = gas_in_block;
                    }
                    block_start = index;
                    gas_in_block = 0;
                    index += 1;
                }
                OpType::JumpDest => {
                    unsafe {
                        jumps.get_unchecked_mut(index).is_jumpdest = true;
                    }
                    unsafe {
                        jumps.get_unchecked_mut(block_start).gas_block = gas_in_block;
                    }
                    block_start = index;
                    gas_in_block = 0;
                    index += 1;
                }
                OpType::Push => {
                    index += ((opcode - opcode::PUSH1) + 2) as usize;
                }
            }
        }
        if gas_in_block != 0 {
            jumps[block_start].gas_block = gas_in_block;
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

    /// Returns true if the valids list is empty

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
