use crate::{collection::vec::Vec, CallContext, ExitError, ExitReason, ExitSucceed};
use bytes::Bytes;
use primitive_types::{H160, U256};

use crate::opcode::OpCode;

pub struct Contract {
    /// Contracts data
    pub input: Bytes,
    /// Contract code
    pub code: Bytes,
    /// Contract address
    pub address: H160,
    /// Caller of the EVM.
    pub caller: H160,
    /// Value send to contract.
    pub value: U256,
    /// Precomputed valid jump addresses
    jumpdest: ValidJumpAddress,
}

impl Contract {
    pub fn new(input: Bytes, code: Bytes, address: H160, caller: H160, value: U256) -> Self {
        let jumpdest = ValidJumpAddress::new(code.as_ref());
        Self {
            input,
            code,
            address,
            caller,
            value,
            jumpdest,
        }
    }

    pub fn is_valid_jump(&self, possition: usize) -> bool {
        self.jumpdest.is_valid(possition)
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

    pub fn opcode(&self, program_counter: usize) -> Result<OpCode, ExitReason> {
        let opcode = {
            if let Some(opcode_byte) = self.code.get(program_counter) {
                let opcode = OpCode::try_from_u8(*opcode_byte);
                // if there is no opcode in code or OpCode is invalid, return error.
                if opcode.is_none() {
                    return Err(ExitError::OpcodeNotFound.into()); // TODO this not seems right, for invalid opcode
                }
                opcode.unwrap()
            } else {
                return Err(ExitSucceed::Stopped.into());
            }
        };
        Ok(opcode)
    }
}

/// Mapping of valid jump destination from code.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidJumpAddress(Vec<bool>);

impl ValidJumpAddress {
    /// Create a new valid mapping from given code bytes.
    pub fn new(code: &[u8]) -> Self {
        let mut jumps: Vec<bool> = Vec::with_capacity(code.len());
        jumps.resize(code.len(), false);

        let mut i = 0;
        while i < code.len() {
            let opcode = code[i] as u8;
            if opcode == OpCode::JUMPDEST as u8 {
                jumps[i] = true;
                i += 1;
            } else if let Some(v) = OpCode::is_push(opcode) {
                i += v as usize + 1;
            } else {
                i += 1;
            }
        }

        Self(jumps)
    }

    /// Get the length of the valid mapping. This is the same as the
    /// code bytes.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns true if the valids list is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns `true` if the position is a valid jump destination. If
    /// not, returns `false`.
    pub fn is_valid(&self, position: usize) -> bool {
        if position >= self.0.len() {
            return false;
        }

        self.0[position]
    }
}
