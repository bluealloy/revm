pub use crate::primitives::CreateScheme;
use crate::primitives::{Bytes, B160, U256};

/// Inputs for a call.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CallInputs {
    /// The target of the call.
    pub contract: B160,
    /// The transfer, if any, in this call.
    pub transfer: Transfer,
    /// The call data of the call.
    #[cfg_attr(
        feature = "serde",
        serde(with = "crate::primitives::utilities::serde_hex_bytes")
    )]
    pub input: Bytes,
    /// The gas limit of the call.
    pub gas_limit: u64,
    /// The context of the call.
    pub context: CallContext,
    /// Is static call
    pub is_static: bool,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CreateInputs {
    pub caller: B160,
    pub scheme: CreateScheme,
    pub value: U256,
    #[cfg_attr(
        feature = "serde",
        serde(with = "crate::primitives::utilities::serde_hex_bytes")
    )]
    pub init_code: Bytes,
    pub gas_limit: u64,
}

/// Call schemes.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
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

/// CallContext of the runtime.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CallContext {
    /// Execution address.
    pub address: B160,
    /// Caller of the EVM.
    pub caller: B160,
    /// The address the contract code was loaded from, if any.
    pub code_address: B160,
    /// Apparent value of the EVM.
    pub apparent_value: U256,
    /// The scheme used for the call.
    pub scheme: CallScheme,
}

impl Default for CallContext {
    fn default() -> Self {
        CallContext {
            address: B160::default(),
            caller: B160::default(),
            code_address: B160::default(),
            apparent_value: U256::default(),
            scheme: CallScheme::Call,
        }
    }
}

/// Transfer from source to target, with given value.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Transfer {
    /// Source address.
    pub source: B160,
    /// Target address.
    pub target: B160,
    /// Transfer value.
    pub value: U256,
}

#[derive(Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SelfDestructResult {
    pub had_value: bool,
    pub target_exists: bool,
    pub is_cold: bool,
    pub previously_destroyed: bool,
}
