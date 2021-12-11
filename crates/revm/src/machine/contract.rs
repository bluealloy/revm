use crate::{alloc::vec::Vec, opcode::opcode_info, CallContext};
use bytes::Bytes;
use primitive_types::{H160, U256};

use crate::instructions::opcode::{self, OpCode};

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
pub enum Analazis {
    JumpDest,
    GasBlockEnd, //contains gas for next block
    None,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AnalazisData {
    pub analazis: Analazis,
    pub gas_block: u64,
}

impl AnalazisData {
    pub fn none() -> Self {
        AnalazisData {
            analazis: Analazis::None,
            gas_block: 0,
        }
    }

    pub fn jump_dest() -> Self {
        AnalazisData {
            analazis: Analazis::JumpDest,
            gas_block: 0,
        }
    }

    pub fn gas_block_end() -> Self {
        AnalazisData {
            analazis: Analazis::GasBlockEnd,
            gas_block: 0,
        }
    }

    pub fn is_jump_dest(&self) -> bool {
        self.analazis == Analazis::JumpDest
    }
}

impl Contract {
    pub fn new(input: Bytes, code: Bytes, address: H160, caller: H160, value: U256) -> Self {
        let code_size = code.len();
        let (jumpdest, code) = Self::analize(code.as_ref());

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
    fn analize(code: &[u8]) -> (ValidJumpAddress, Vec<u8>) {
        let mut jumps: Vec<AnalazisData> = Vec::with_capacity(code.len());
        jumps.resize(code.len(), AnalazisData::none());
        let opcode_info = opcode_info();
        let mut i = 0;
        let mut first_gas_block: Option<u64> = None;
        let mut block_start: usize = 0;
        let mut gas_in_block: u64 = 0;
        while i < code.len() {
            let opcode = code[i] as u8;
            let info = &opcode_info[opcode as usize];
            gas_in_block += info.gas;

            if info.gas_block_end {
                if first_gas_block.is_some() {
                    jumps[block_start].gas_block = gas_in_block;
                } else {
                    first_gas_block = Some(gas_in_block);
                }
                block_start = i;
                gas_in_block = 0;
            }
            if opcode == opcode::JUMPDEST as u8 {
                jumps[i].analazis = Analazis::JumpDest;
            } else if let Some(v) = OpCode::is_push(opcode) {
                i += v as usize;
            }
            i += 1;
        }
        if gas_in_block != 0 {
            if first_gas_block.is_some() {
                jumps[block_start].gas_block = gas_in_block;
            } else {
                first_gas_block = Some(gas_in_block);
            }
        }
        let padding = i - code.len();
        // +1 is for forced STOP opcode at the end of contract, it is precausion
        // if there is none, and if there is STOP our additional opcode will do nothing.
        jumps.resize(jumps.len() + padding + 1, AnalazisData::none());
        let mut code = code.to_vec();
        code.resize(code.len() + padding + 1, 0);

        (
            ValidJumpAddress::new(jumps, first_gas_block.unwrap_or_default()),
            code,
        )
    }

    #[inline(always)]
    pub fn is_valid_jump(&self, possition: usize) -> bool {
        self.jumpdest.is_valid(possition)
    }

    #[inline(always)]
    pub fn gas_block(&self, possition: usize) -> u64 {
        self.jumpdest.gas_block(possition)
    }
    pub fn first_gas_block(&self) -> u64 {
        self.jumpdest.first_gas_block
    }

    pub fn new_with_context(input: Bytes, code: Bytes, call_context: &CallContext) -> Self {
        Self::new(
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
    analazis: Vec<AnalazisData>,
}

impl ValidJumpAddress {
    pub fn new(analazis: Vec<AnalazisData>, first_gas_block: u64) -> Self {
        Self {
            analazis,
            first_gas_block,
        }
    }
    /// Get the length of the valid mapping. This is the same as the
    /// code bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.analazis.len()
    }

    /// Returns true if the valids list is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns `true` if the position is a valid jump destination. If
    /// not, returns `false`.
    #[inline(always)]
    pub fn is_valid(&self, position: usize) -> bool {
        if position >= self.analazis.len() {
            return false;
        }

        self.analazis[position].is_jump_dest()
    }

    #[inline(always)]
    pub fn gas_block(&self, position: usize) -> u64 {
        self.analazis[position].gas_block
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn analize_padding_dummy() {
        //let (_, padding) = Contract::analize(&[opcode::CODESIZE, opcode::PUSH1, 0x00]);
        //assert_eq!(padding.len(), 0, "Padding should be zero");
    }
    #[test]
    fn analize_padding_two_missing() {
        //let (_, padding) = Contract::analize(&[opcode::CODESIZE, opcode::PUSH3, 0x00]);
        //assert_eq!(padding, 2, "Padding should be zero");
    }
}
