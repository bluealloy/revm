use super::bytecode::{Bytecode, BytecodeLocked};
use crate::{alloc::vec::Vec, CallContext, Spec};
use bytes::Bytes;
use primitive_types::H160;
use ruint::aliases::U256;
use std::sync::Arc;

pub struct Contract {
    /// Contracts data
    pub input: Bytes,
    /// Bytecode contains contract code, size of original code, analysis with gas block and jump table.
    /// Note that current code is extended with push padding and STOP at end.
    pub bytecode: BytecodeLocked,
    /// Contract address
    pub address: H160,
    /// Caller of the EVM.
    pub caller: H160,
    /// Value send to contract.
    pub value: U256,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Analysis {
    JumpDest,
    GasBlockEnd, //contains gas for next block
    None,
}

const JUMP_MASK: u32 = 0x80000000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AnalysisData {
    /// This variable packs two informations:
    /// IS_JUMP (1bit) | gas block ( 31bits)
    is_jump_and_gas_block: u32,
}

impl AnalysisData {
    pub fn none() -> Self {
        AnalysisData {
            is_jump_and_gas_block: 0,
        }
    }

    pub fn set_is_jump(&mut self) {
        self.is_jump_and_gas_block |= JUMP_MASK;
    }

    pub fn set_gas_block(&mut self, gas_block: u32) {
        let jump = self.is_jump_and_gas_block & JUMP_MASK;
        self.is_jump_and_gas_block = gas_block | jump;
    }

    pub fn is_jump(&self) -> bool {
        self.is_jump_and_gas_block & JUMP_MASK == JUMP_MASK
    }

    pub fn gas_block(&self) -> u64 {
        (self.is_jump_and_gas_block & (!JUMP_MASK)) as u64
    }
}

impl Contract {
    pub fn new<SPEC: Spec>(
        input: Bytes,
        bytecode: Bytecode,
        address: H160,
        caller: H160,
        value: U256,
    ) -> Self {
        let bytecode = bytecode.lock::<SPEC>();
        Self {
            input,
            bytecode,
            address,
            caller,
            value,
        }
    }

    pub fn is_valid_jump(&self, possition: usize) -> bool {
        self.bytecode.jumptable().is_valid(possition)
    }

    pub fn gas_block(&self, possition: usize) -> u64 {
        self.bytecode.jumptable().gas_block(possition)
    }
    pub fn first_gas_block(&self) -> u64 {
        self.bytecode.jumptable().first_gas_block as u64
    }

    pub fn new_with_context<SPEC: Spec>(
        input: Bytes,
        bytecode: Bytecode,
        call_context: &CallContext,
    ) -> Self {
        Self::new::<SPEC>(
            input,
            bytecode,
            call_context.address,
            call_context.caller,
            call_context.apparent_value,
        )
    }
}

/// Mapping of valid jump destination from code.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ValidJumpAddress {
    pub first_gas_block: u32,
    /// Rc is used here so that we dont need to copy vector. We can move it to more suitable more accessable structure
    /// without copying underlying vec.
    pub analysis: Arc<Vec<AnalysisData>>,
}

impl ValidJumpAddress {
    pub fn new(analysis: Arc<Vec<AnalysisData>>, first_gas_block: u32) -> Self {
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
        self.analysis[position].is_jump()
    }

    pub fn gas_block(&self, position: usize) -> u64 {
        self.analysis[position].gas_block()
    }
}

#[cfg(test)]
mod tests {
    use super::AnalysisData;

    #[test]
    pub fn test_jump_set() {
        let mut jump = AnalysisData::none();
        assert!(!jump.is_jump());
        assert_eq!(jump.gas_block(), 0);

        jump.set_gas_block(2350);
        assert!(!jump.is_jump());
        assert_eq!(jump.gas_block(), 2350);

        jump.set_is_jump();
        assert!(jump.is_jump());
        assert_eq!(jump.gas_block(), 2350);

        jump.set_gas_block(10);
        assert!(jump.is_jump());
        assert_eq!(jump.gas_block(), 10);

        jump.set_gas_block(350);
        assert!(jump.is_jump());
        assert_eq!(jump.gas_block(), 350);
    }
}
