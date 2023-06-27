use super::analysis::{to_analysed, BytecodeLocked};
use crate::primitives::{Bytecode, Bytes, B160, U256};
use crate::CallContext;
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

impl Contract {
    pub fn new(input: Bytes, bytecode: Bytecode, address: B160, caller: B160, value: U256) -> Self {
        let bytecode = to_analysed(bytecode).try_into().expect("it is analyzed");

        Self {
            input,
            bytecode,
            address,
            caller,
            value,
        }
    }

    /// Create new contract from environment
    pub fn new_env(env: &Env, bytecode: Bytecode) -> Self {
        let contract_address = match env.tx.transact_to {
            TransactTo::Call(caller) => caller,
            TransactTo::Create(..) => B160::zero(),
        };
        Self::new(
            env.tx.data.clone(),
            bytecode,
            contract_address,
            env.tx.caller,
            env.tx.value,
        )
    }

    pub fn is_valid_jump(&self, possition: usize) -> bool {
        self.bytecode.jump_map().is_valid(possition)
    }

    pub fn new_with_context(input: Bytes, bytecode: Bytecode, call_context: &CallContext) -> Self {
        Self::new(
            input,
            bytecode,
            call_context.address,
            call_context.caller,
            call_context.apparent_value,
        )
    }
}
