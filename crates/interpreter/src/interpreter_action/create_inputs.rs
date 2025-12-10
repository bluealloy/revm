use context_interface::CreateScheme;
use core::cell::OnceCell;
use primitives::{Address, Bytes, U256};

/// Inputs for a create call
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CreateInputs {
    /// Caller address of the EVM
    pub caller: Address,
    /// The create scheme
    pub scheme: CreateScheme,
    /// The value to transfer
    pub value: U256,
    /// The init code of the contract
    pub init_code: Bytes,
    /// The gas limit of the call
    pub gas_limit: u64,
    /// Cached created address. This is computed lazily and cached to avoid
    /// redundant keccak computations when inspectors call `created_address`.
    #[cfg_attr(feature = "serde", serde(skip))]
    cached_address: OnceCell<Address>,
}

impl CreateInputs {
    /// Creates a new `CreateInputs` instance.
    pub fn new(
        caller: Address,
        scheme: CreateScheme,
        value: U256,
        init_code: Bytes,
        gas_limit: u64,
    ) -> Self {
        Self {
            caller,
            scheme,
            value,
            init_code,
            gas_limit,
            cached_address: OnceCell::new(),
        }
    }

    /// Returns the address that this create call will create.
    ///
    /// The result is cached to avoid redundant keccak computations.
    pub fn created_address(&self, nonce: u64) -> Address {
        *self.cached_address.get_or_init(|| match self.scheme {
            CreateScheme::Create => self.caller.create(nonce),
            CreateScheme::Create2 { salt } => self
                .caller
                .create2_from_code(salt.to_be_bytes(), &self.init_code),
            CreateScheme::Custom { address } => address,
        })
    }
}
