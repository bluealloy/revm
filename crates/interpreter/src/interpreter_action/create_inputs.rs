use primitives::{Address, Bytes, TxKind, U256};
use std::boxed::Box;
use wiring::{default::CreateScheme, Transaction};

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
    /// Creates new create inputs.
    pub fn new(tx_env: &impl Transaction, gas_limit: u64) -> Option<Self> {
        let TxKind::Create = tx_env.kind() else {
            return None;
        };

        Some(CreateInputs {
            caller: *tx_env.caller(),
            scheme: CreateScheme::Create,
            value: *tx_env.value(),
            init_code: tx_env.data().clone(),
            gas_limit,
        })
    }

    /// Returns boxed create inputs.
    pub fn new_boxed(tx_env: &impl Transaction, gas_limit: u64) -> Option<Box<Self>> {
        Self::new(tx_env, gas_limit).map(Box::new)
    }

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
