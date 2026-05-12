use context_interface::CreateScheme;
use core::cell::OnceCell;
use primitives::{keccak256, Address, Bytes, B256, U256};

/// Inputs for a create call
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CreateInputs {
    /// Caller address of the EVM
    caller: Address,
    /// The create scheme
    scheme: CreateScheme,
    /// The value to transfer
    value: U256,
    /// The init code of the contract
    init_code: Bytes,
    /// The gas limit of the call
    gas_limit: u64,
    /// State gas reservoir (EIP-8037). Passed from parent frame to child frame.
    reservoir: u64,
    /// Cached created address. This is computed lazily and cached to avoid
    /// redundant keccak computations when inspectors call `created_address`.
    #[cfg_attr(feature = "serde", serde(skip))]
    cached_address: OnceCell<Address>,
    /// Cached init code hash. Shared between `created_address()` (for CREATE2)
    /// and frame initialization (for `ExtBytecode`), ensuring keccak256 of the
    /// init code is computed at most once.
    #[cfg_attr(feature = "serde", serde(skip))]
    cached_init_code_hash: OnceCell<B256>,
}

impl CreateInputs {
    /// Creates a new `CreateInputs` instance.
    pub const fn new(
        caller: Address,
        scheme: CreateScheme,
        value: U256,
        init_code: Bytes,
        gas_limit: u64,
        reservoir: u64,
    ) -> Self {
        Self {
            caller,
            scheme,
            value,
            init_code,
            gas_limit,
            reservoir,
            cached_address: OnceCell::new(),
            cached_init_code_hash: OnceCell::new(),
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
                .create2(salt.to_be_bytes(), self.init_code_hash()),
            CreateScheme::Custom { address } => address,
        })
    }

    /// Returns the keccak256 hash of the init code.
    ///
    /// The result is cached so that `created_address()` and frame initialization
    /// share a single hash computation.
    pub fn init_code_hash(&self) -> B256 {
        *self
            .cached_init_code_hash
            .get_or_init(|| keccak256(self.init_code.as_ref()))
    }

    /// Returns the caller address of the EVM.
    pub const fn caller(&self) -> Address {
        self.caller
    }

    /// Returns the create scheme of the EVM.
    pub const fn scheme(&self) -> CreateScheme {
        self.scheme
    }

    /// Returns the value to transfer.
    pub const fn value(&self) -> U256 {
        self.value
    }

    /// Returns the init code of the contract.
    pub const fn init_code(&self) -> &Bytes {
        &self.init_code
    }

    /// Returns the gas limit of the call.
    pub const fn gas_limit(&self) -> u64 {
        self.gas_limit
    }

    /// Set call
    pub const fn set_call(&mut self, caller: Address) {
        self.caller = caller;
        self.cached_address = OnceCell::new();
    }

    /// Set scheme
    pub const fn set_scheme(&mut self, scheme: CreateScheme) {
        self.scheme = scheme;
        self.cached_address = OnceCell::new();
    }

    /// Set value
    pub const fn set_value(&mut self, value: U256) {
        self.value = value;
    }

    /// Set init code
    pub fn set_init_code(&mut self, init_code: Bytes) {
        self.init_code = init_code;
        self.cached_address = OnceCell::new();
        self.cached_init_code_hash = OnceCell::new();
    }

    /// Set gas limit
    pub const fn set_gas_limit(&mut self, gas_limit: u64) {
        self.gas_limit = gas_limit;
    }

    /// Returns the state gas reservoir (EIP-8037).
    pub const fn reservoir(&self) -> u64 {
        self.reservoir
    }

    /// Set the state gas reservoir (EIP-8037).
    pub const fn set_reservoir(&mut self, reservoir: u64) {
        self.reservoir = reservoir;
    }
}
