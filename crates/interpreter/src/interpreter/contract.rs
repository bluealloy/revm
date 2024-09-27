use crate::CallInputs;
use bytecode::Bytecode;
use primitives::{Address, Bytes, TxKind, B256, U256};
use wiring::{default::EnvWiring, EvmWiring, Transaction};

/// EVM contract information.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Contract {
    /// Contracts data
    pub input: Bytes,
    /// Bytecode contains contract code, size of original code, analysis with gas block and jump table.
    /// Note that current code is extended with push padding and STOP at end.
    pub bytecode: Bytecode,
    /// Bytecode hash for legacy. For EOF this would be None.
    pub hash: Option<B256>,
    /// Target address of the account. Storage of this address is going to be modified.
    pub target_address: Address,
    /// Address of the account the bytecode was loaded from. This can be different from target_address
    /// in the case of DELEGATECALL or CALLCODE
    pub bytecode_address: Option<Address>,
    /// Caller of the EVM.
    pub caller: Address,
    /// Value send to contract from transaction or from CALL opcodes.
    pub call_value: U256,
}

impl Contract {
    /// Instantiates a new contract by analyzing the given bytecode.
    #[inline]
    pub fn new(
        input: Bytes,
        bytecode: Bytecode,
        hash: Option<B256>,
        target_address: Address,
        bytecode_address: Option<Address>,
        caller: Address,
        call_value: U256,
    ) -> Self {
        let bytecode = bytecode.into_analyzed();

        Self {
            input,
            bytecode,
            hash,
            target_address,
            bytecode_address,
            caller,
            call_value,
        }
    }

    /// Creates a new contract from the given [`EnvWiring`].
    #[inline]
    pub fn new_env<EvmWiringT: EvmWiring>(
        env: &EnvWiring<EvmWiringT>,
        bytecode: Bytecode,
        hash: Option<B256>,
    ) -> Self {
        let bytecode_address = match env.tx.kind() {
            TxKind::Call(caller) => Some(caller),
            TxKind::Create => None,
        };
        let target_address = bytecode_address.unwrap_or_default();

        Self::new(
            env.tx.common_fields().input().clone(),
            bytecode,
            hash,
            target_address,
            bytecode_address,
            env.tx.common_fields().caller(),
            env.tx.common_fields().value(),
        )
    }

    /// Creates a new contract from the given inputs.
    #[inline]
    pub fn new_with_context(
        input: Bytes,
        bytecode: Bytecode,
        hash: Option<B256>,
        call_context: &CallInputs,
    ) -> Self {
        Self::new(
            input,
            bytecode,
            hash,
            call_context.target_address,
            Some(call_context.bytecode_address),
            call_context.caller,
            call_context.call_value(),
        )
    }

    /// Returns whether the given position is a valid jump destination.
    #[inline]
    pub fn is_valid_jump(&self, pos: usize) -> bool {
        self.bytecode
            .legacy_jump_table()
            .map(|i| i.is_valid(pos))
            .unwrap_or(false)
    }
}
