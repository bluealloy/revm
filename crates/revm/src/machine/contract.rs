use crate::{alloc::vec::Vec, opcode::OPCODE_INFO, CallContext};
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
        let (jumpdest, padding) = Self::analize(code.as_ref());

        let mut code = code.to_vec();
        let code_size = code.len();
        if padding != 0 {
            code.resize(code.len() + padding + 1, 0);
        } else {
            code.resize(code.len() + 1, 0);
        };
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
    /// TODO probably can optimize few things
    fn analize(code: &[u8]) -> (ValidJumpAddress, usize) {
        let mut jumps: Vec<AnalazisData> = Vec::with_capacity(code.len());
        jumps.resize(code.len(), AnalazisData::none());
        let mut is_push_last = false;
        let mut i = 0;
        let opcode_info = OPCODE_INFO();
        let mut gas_block: u64 = 0;
        let mut block_start = 0;
        while i < code.len() {
            let opcode = code[i] as u8;
            let info = &opcode_info[opcode as usize];
            gas_block = gas_block.saturating_add(info.gas);

            if opcode == opcode::JUMPDEST as u8 {
                is_push_last = false;
                jumps[i] = AnalazisData::jump_dest();
                i += 1;
            } else if let Some(v) = OpCode::is_push(opcode) {
                is_push_last = true;
                i += v as usize + 1;
            } else {
                is_push_last = false;
                i += 1;
            }
            if info.gas_block_end {
                jumps[block_start].gas_block = gas_block;
                block_start = i;
                gas_block = 0;
            }
        }
        let padding = if is_push_last { i - code.len() } else { 0 };
        jumps.resize(jumps.len() + padding, AnalazisData::none());
        if jumps.len() == 0 {
            // for usecase when contract is empty
            jumps.resize(1,AnalazisData::none());
        }
        (ValidJumpAddress::new(jumps), padding)
    }

    #[inline(always)]
    pub fn is_valid_jump(&self, possition: usize) -> bool {
        self.jumpdest.is_valid(possition)
    }

    #[inline(always)]
    pub fn gas_block(&self, possition: usize) -> u64 {
        self.jumpdest.gas_block(possition)
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
    analazis: Vec<AnalazisData>,
}

impl ValidJumpAddress {
    pub fn new(analazis: Vec<AnalazisData>) -> Self {
        Self { analazis }
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
        let (_, padding) = Contract::analize(&[opcode::CODESIZE, opcode::PUSH1, 0x00]);
        assert_eq!(padding, 0, "Padding should be zero");
    }
    #[test]
    fn analize_padding_two_missing() {
        let (_, padding) = Contract::analize(&[opcode::CODESIZE, opcode::PUSH3, 0x00]);
        assert_eq!(padding, 2, "Padding should be zero");
    }
}
