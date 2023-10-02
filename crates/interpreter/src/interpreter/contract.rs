use super::analysis::{to_analysed, BytecodeLocked};
use crate::primitives::{Address, Bytecode, Bytes, Env, TransactTo, B256, U256};
use crate::CallContext;

#[derive(Clone, Debug, Default)]
pub struct Contract {
    /// Contracts data
    pub input: Bytes,
    /// Bytecode contains contract code, size of original code, analysis with gas block and jump table.
    /// Note that current code is extended with push padding and STOP at end.
    pub bytecode: BytecodeLocked,
    /// Bytecode hash.
    pub hash: B256,
    /// Contract address
    pub address: Address,
    /// Caller of the EVM.
    pub caller: Address,
    /// Value send to contract.
    pub value: U256,
}

impl Contract {
    /// Instantiates a new contract by analyzing the given bytecode.
    #[inline]
    pub fn new(
        input: Bytes,
        bytecode: Bytecode,
        hash: B256,
        address: Address,
        caller: Address,
        value: U256,
    ) -> Self {
        let bytecode = to_analysed(bytecode).try_into().expect("it is analyzed");

        Self {
            input,
            bytecode,
            hash,
            address,
            caller,
            value,
        }
    }

    /// Creates a new contract from the given [`Env`].
    #[inline]
    pub fn new_env(env: &Env, bytecode: Bytecode, hash: B256) -> Self {
        let contract_address = match env.tx.transact_to {
            TransactTo::Call(caller) => caller,
            TransactTo::Create(..) => Address::ZERO,
        };
        Self::new(
            env.tx.data.clone(),
            bytecode,
            hash,
            contract_address,
            env.tx.caller,
            env.tx.value,
        )
    }

    /// Creates a new contract from the given [`CallContext`].
    #[inline]
    pub fn new_with_context(
        input: Bytes,
        bytecode: Bytecode,
        hash: B256,
        call_context: &CallContext,
    ) -> Self {
        Self::new(
            input,
            bytecode,
            hash,
            call_context.address,
            call_context.caller,
            call_context.apparent_value,
        )
    }

    /// Returns whether the given position is a valid jump destination.
    #[inline]
    pub fn is_valid_jump(&self, pos: usize) -> bool {
        self.bytecode.jump_map().is_valid(pos)
    }
}
