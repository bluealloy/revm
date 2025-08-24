use context_interface::CreateScheme;
use primitives::{Address, AddressAndId, Bytes, U256};

/// Inputs for a create call
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CreateInputs {
    /// Caller address of the EVM
    pub caller: AddressAndId,
    /// The create scheme
    pub scheme: CreateScheme,
    /// The value to transfer
    pub value: U256,
    /// The init code of the contract
    pub init_code: Bytes,
    /// The gas limit of the call
    pub gas_limit: u64,
}

impl CreateInputs {
    /// Returns the address that this create call will create.
    pub fn created_address(&self, nonce: u64) -> Address {
        match self.scheme {
            CreateScheme::Create => self.caller.address().create(nonce),
            CreateScheme::Create2 { salt } => self
                .caller
                .address()
                .create2_from_code(salt.to_be_bytes(), &self.init_code),
            CreateScheme::Custom { address } => address,
        }
    }
}
