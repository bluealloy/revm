use super::analysis::to_analysed;
use crate::{
    primitives::{Address, Bytecode, Bytes, Env, B256, U256},
    CallInputs,
};
use revm_primitives::TxKind;

/// EVM contract information.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Contract {
    /// Contracts data
    pub input: Bytes,
    /// Bytecode contains contract code, size of original code, analysis with gas block and jump
    /// table.
    /// Note that current code is extended with push padding and STOP at the end.
    pub bytecode: Bytecode,
    /// Bytecode hash for legacy. For EOF this would be None.
    pub hash: Option<B256>,
    /// Target address of the account. Storage of this address is going to be modified.
    pub target_address: Address,
    /// Address of the account the bytecode was loaded from. This can be different from
    /// target_address in the case of DELEGATECALL or CALLCODE
    pub bytecode_address: Option<Address>,
    /// Caller of the EVM.
    pub caller: Address,
    /// Value sent to contract from transaction or from CALL opcodes.
    pub call_value: U256,
    /// An address of EIP-7702 resolved proxy.
    /// We should store this information because it doesn't
    /// always match to bytecode address.
    /// Especially when we have DELEGATE or CALLCODE proxied though EIP-7702.
    pub eip7702_address: Option<Address>,
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
        let bytecode = to_analysed(bytecode);

        Self {
            input,
            bytecode,
            hash,
            target_address,
            bytecode_address,
            eip7702_address: None,
            caller,
            call_value,
        }
    }

    /// Creates a new contract from the given [`Env`].
    #[inline]
    pub fn new_env(env: &Env, bytecode: Bytecode, hash: Option<B256>) -> Self {
        let contract_address = match env.tx.transact_to {
            TxKind::Call(caller) => caller,
            TxKind::Create => Address::ZERO,
        };
        let bytecode_address = match env.tx.transact_to {
            TxKind::Call(caller) => Some(caller),
            TxKind::Create => None,
        };
        Self::new(
            env.tx.data.clone(),
            bytecode,
            hash,
            contract_address,
            bytecode_address,
            env.tx.caller,
            env.tx.value,
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

    #[inline]
    pub fn effective_bytecode_address(&self) -> Address {
        self.eip7702_address
            .unwrap_or_else(|| self.bytecode_address())
    }

    #[inline]
    pub fn bytecode_address(&self) -> Address {
        self.bytecode_address.unwrap_or(self.target_address)
    }
}
