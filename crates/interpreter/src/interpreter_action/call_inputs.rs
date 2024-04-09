use crate::primitives::{Address, Bytes, TransactTo, TxEnv, U256};
use core::ops::Range;
use std::boxed::Box;

/// Inputs for a call.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CallInputs {
    /// The call data of the call.
    pub input: Bytes,
    /// The return memory offset where the output of the call is written.
    /// For EOF this range is invalid as EOF does write output to memory.
    pub return_memory_offset: Range<usize>,
    /// The gas limit of the call.
    pub gas_limit: u64,
    /// The account address of bytecode that is going to be executed.  
    pub bytecode_address: Address,
    /// Target address, this account storage is going to be modified.
    pub target_address: Address,
    /// This caller is invoking the call.
    pub caller: Address,
    /// Value that is transferred in Ether.
    ///
    /// If enum is [`TransferValue::Value`] balance is transferer from `caller` to the `target_address`.
    ///
    /// If enum is [`TransferValue::ApparentValue`] balance transfer is **not**
    /// done and apparent value is used by CALLVALUE opcode. Used by delegate call.
    pub value: TransferValue,
    /// The scheme used for the call. Call, callcode, delegatecall or staticcall.
    pub scheme: CallScheme,
    /// Whether this is a static call.
    pub is_static: bool,
    /// Call is initiated from EOF bytecode.
    pub is_eof: bool,
}

impl CallInputs {
    /// Creates new call inputs.
    pub fn new(tx_env: &TxEnv, gas_limit: u64) -> Option<Self> {
        let TransactTo::Call(target_address) = tx_env.transact_to else {
            return None;
        };

        Some(CallInputs {
            input: tx_env.data.clone(),
            gas_limit,
            target_address,
            bytecode_address: target_address,
            caller: tx_env.caller,
            value: TransferValue::Value(tx_env.value),
            scheme: CallScheme::Call,
            is_static: false,
            is_eof: false,
            return_memory_offset: 0..0,
        })
    }

    /// Returns boxed call inputs.
    pub fn new_boxed(tx_env: &TxEnv, gas_limit: u64) -> Option<Box<Self>> {
        Self::new(tx_env, gas_limit).map(Box::new)
    }

    /// Return call value
    pub fn call_value(&self) -> U256 {
        let (TransferValue::Value(value) | TransferValue::ApparentValue(value)) = self.value;
        value
    }
}

/// Call schemes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CallScheme {
    /// `CALL`.
    Call,
    /// `CALLCODE`
    CallCode,
    /// `DELEGATECALL`
    DelegateCall,
    /// `STATICCALL`
    StaticCall,
}

/// Transfered value from caller to callee.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TransferValue {
    /// Transfer value from caller to callee.
    Value(U256),
    /// For delegate call, the value is not transferred but
    /// apparent value is used for CALLVALUE opcode
    ApparentValue(U256),
}

impl Default for TransferValue {
    fn default() -> Self {
        TransferValue::Value(U256::ZERO)
    }
}
