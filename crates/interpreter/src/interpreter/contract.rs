use super::analysis::{to_analysed, BytecodeLocked};
use crate::primitives::{Bytecode, Spec, B160, U256};
use crate::CallContext;
use bytes::Bytes;
use revm_primitives::{Env, TransactTo};

#[derive(Clone, Default)]
pub struct Contract {
    /// Contracts data
    pub input: Bytes,
    /// Bytecode contains contract code, size of original code, analysis with gas block and jump table.
    /// Note that current code is extended with push padding and STOP at end.
    pub bytecode: BytecodeLocked,
    /// Contract address
    pub address: B160,
    /// Caller of the EVM.
    pub caller: B160,
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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
        address: B160,
        caller: B160,
        value: U256,
    ) -> Self {
        let bytecode = to_analysed::<SPEC>(bytecode)
            .try_into()
            .expect("it is analyzed");

        Self {
            input,
            bytecode,
            address,
            caller,
            value,
        }
    }

    /// Create new contract from environment
    /// TODO: Add spec related match to analyze bytecode by env.cfg.spec_id variable
    pub fn new_env<SPEC: Spec>(env: &Env, bytecode: Bytecode) -> Self {
        let contract_address = match env.tx.transact_to {
            TransactTo::Call(caller) => caller,
            TransactTo::Create(..) => B160::zero(),
        };
        Self::new::<SPEC>(
            env.tx.data.clone(),
            bytecode,
            contract_address,
            env.tx.caller,
            env.tx.value,
        )
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
