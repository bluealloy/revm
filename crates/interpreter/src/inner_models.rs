pub use crate::primitives::CreateScheme;
use crate::primitives::{Address, Bytes, U256};

/// Inputs for a call.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CallInputs {
    /// The target of the call.
    pub contract: Address,
    /// The transfer, if any, in this call.
    pub transfer: Transfer,
    /// The call data of the call.
    pub input: Bytes,
    /// The gas limit of the call.
    pub gas_limit: u64,
    /// The context of the call.
    pub context: CallContext,
    /// Whether this is a static call.
    pub is_static: bool,
}

/// Inputs for a create call.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CreateInputs {
    /// Caller address of the EVM.
    pub caller: Address,
    /// The create scheme.
    pub scheme: CreateScheme,
    /// The value to transfer.
    pub value: U256,
    /// The init code of the contract.
    pub init_code: Bytes,
    /// The gas limit of the call.
    pub gas_limit: u64,
}

impl CreateInputs {
    /// Returns the address that this create call will create.
    pub fn created_address(&self, nonce: u64) -> Address {
        match self.scheme {
            CreateScheme::Create => self.caller.create(nonce),
            CreateScheme::Create2 { salt } => self
                .caller
                .create2_from_code(salt.to_be_bytes(), &self.init_code),
        }
    }
}

/// Call schemes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CallScheme {
    /// `CALL`
    Call,
    /// `CALLCODE`
    CallCode,
    /// `DELEGATECALL`
    DelegateCall,
    /// `STATICCALL`
    StaticCall,
}

/// Context of a runtime call.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CallContext {
    /// Execution address.
    pub address: Address,
    /// Caller address of the EVM.
    pub caller: Address,
    /// The address the contract code was loaded from, if any.
    pub code_address: Address,
    /// Apparent value of the EVM.
    pub apparent_value: U256,
    /// The scheme used for the call.
    pub scheme: CallScheme,
}

impl Default for CallContext {
    fn default() -> Self {
        CallContext {
            address: Address::default(),
            caller: Address::default(),
            code_address: Address::default(),
            apparent_value: U256::default(),
            scheme: CallScheme::Call,
        }
    }
}

/// Transfer from source to target, with given value.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Transfer {
    /// The source address.
    pub source: Address,
    /// The target address.
    pub target: Address,
    /// The transfer value.
    pub value: U256,
}

/// Result of a call that resulted in a self destruct.
#[derive(Default, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SelfDestructResult {
    pub had_value: bool,
    pub target_exists: bool,
    pub is_cold: bool,
    pub previously_destroyed: bool,
}
