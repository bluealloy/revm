use revm_primitives::Transaction;

use crate::primitives::{Address, Bytes, TxKind, U256};
use core::ops::Range;
use std::boxed::Box;

/// Inputs for a call.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CallInputs {
    /// The call data of the call.
    pub input: Bytes,
    /// The return memory offset where the output of the call is written.
    ///
    /// In EOF, this range is invalid as EOF calls do not write output to memory.
    pub return_memory_offset: Range<usize>,
    /// The gas limit of the call.
    pub gas_limit: u64,
    /// The account address of bytecode that is going to be executed.
    ///
    /// Previously `context.code_address`.
    pub bytecode_address: Address,
    /// Target address, this account storage is going to be modified.
    ///
    /// Previously `context.address`.
    pub target_address: Address,
    /// This caller is invoking the call.
    ///
    /// Previously `context.caller`.
    pub caller: Address,
    /// Call value.
    ///
    /// NOTE: This value may not necessarily be transferred from caller to callee, see [`CallValue`].
    ///
    /// Previously `transfer.value` or `context.apparent_value`.
    pub value: CallValue,
    /// The call scheme.
    ///
    /// Previously `context.scheme`.
    pub scheme: CallScheme,
    /// Whether the call is a static call, or is initiated inside a static call.
    pub is_static: bool,
    /// Whether the call is initiated from EOF bytecode.
    pub is_eof: bool,
}

impl CallInputs {
    /// Creates new call inputs.
    ///
    /// Returns `None` if the transaction is not a call.
    pub fn new(tx_env: &impl Transaction, gas_limit: u64) -> Option<Self> {
        let TxKind::Call(target_address) = tx_env.kind() else {
            return None;
        };
        Some(CallInputs {
            input: tx_env.data().clone(),
            gas_limit,
            target_address,
            bytecode_address: target_address,
            caller: *tx_env.caller(),
            value: CallValue::Transfer(*tx_env.value()),
            scheme: CallScheme::Call,
            is_static: false,
            is_eof: false,
            return_memory_offset: 0..0,
        })
    }

    /// Creates new boxed call inputs.
    ///
    /// Returns `None` if the transaction is not a call.
    pub fn new_boxed(tx_env: &impl Transaction, gas_limit: u64) -> Option<Box<Self>> {
        Self::new(tx_env, gas_limit).map(Box::new)
    }

    /// Returns `true` if the call will transfer a non-zero value.
    #[inline]
    pub fn transfers_value(&self) -> bool {
        self.value.transfer().is_some_and(|x| x > U256::ZERO)
    }

    /// Returns the transfer value.
    ///
    /// This is the value that is transferred from caller to callee, see [`CallValue`].
    #[inline]
    pub const fn transfer_value(&self) -> Option<U256> {
        self.value.transfer()
    }

    /// Returns the **apparent** call value.
    ///
    /// This value is not actually transferred, see [`CallValue`].
    #[inline]
    pub const fn apparent_value(&self) -> Option<U256> {
        self.value.apparent()
    }

    /// Returns the address of the transfer source account.
    ///
    /// This is only meaningful if `transfers_value` is `true`.
    #[inline]
    pub const fn transfer_from(&self) -> Address {
        self.caller
    }

    /// Returns the address of the transfer target account.
    ///
    /// This is only meaningful if `transfers_value` is `true`.
    #[inline]
    pub const fn transfer_to(&self) -> Address {
        self.target_address
    }

    /// Returns the call value, regardless of the transfer value type.
    ///
    /// NOTE: this value may not necessarily be transferred from caller to callee, see [`CallValue`].
    #[inline]
    pub const fn call_value(&self) -> U256 {
        self.value.get()
    }
}

/// Call scheme.
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
    /// `EXTCALL`
    ExtCall,
    /// `EXTSTATICCALL`
    ExtStaticCall,
    /// `EXTDELEGATECALL`
    ExtDelegateCall,
}

impl CallScheme {
    /// Returns true if it is EOF EXT*CALL.
    pub fn is_ext(&self) -> bool {
        matches!(
            self,
            Self::ExtCall | Self::ExtStaticCall | Self::ExtDelegateCall
        )
    }

    /// Returns true if it is ExtDelegateCall.
    pub fn is_ext_delegate_call(&self) -> bool {
        matches!(self, Self::ExtDelegateCall)
    }
}

/// Call value.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CallValue {
    /// Concrete value, transferred from caller to callee at the end of the transaction.
    Transfer(U256),
    /// Apparent value, that is **not** actually transferred.
    ///
    /// Set when in a `DELEGATECALL` call type, and used by the `CALLVALUE` opcode.
    Apparent(U256),
}

impl Default for CallValue {
    #[inline]
    fn default() -> Self {
        CallValue::Transfer(U256::ZERO)
    }
}

impl CallValue {
    /// Returns the call value, regardless of the type.
    #[inline]
    pub const fn get(&self) -> U256 {
        match *self {
            Self::Transfer(value) | Self::Apparent(value) => value,
        }
    }

    /// Returns the transferred value, if any.
    #[inline]
    pub const fn transfer(&self) -> Option<U256> {
        match *self {
            Self::Transfer(transfer) => Some(transfer),
            Self::Apparent(_) => None,
        }
    }

    /// Returns whether the call value will be transferred.
    #[inline]
    pub const fn is_transfer(&self) -> bool {
        matches!(self, Self::Transfer(_))
    }

    /// Returns the apparent value, if any.
    #[inline]
    pub const fn apparent(&self) -> Option<U256> {
        match *self {
            Self::Transfer(_) => None,
            Self::Apparent(apparent) => Some(apparent),
        }
    }

    /// Returns whether the call value is apparent, and not actually transferred.
    #[inline]
    pub const fn is_apparent(&self) -> bool {
        matches!(self, Self::Apparent(_))
    }
}
