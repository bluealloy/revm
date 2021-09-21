use bytes::Bytes;
use primitive_types::{H160, H256, U256};

/// Basic account information.
#[derive(Clone, Copy, Eq, PartialEq, Debug, Default)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Basic {
    /// Account balance.
    pub balance: U256,
    /// Account nonce.
    pub nonce: U256,
}

/// Create scheme.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum CreateScheme {
    /// Legacy create scheme of `CREATE`.
    Legacy {
        /// Caller of the create.
        caller: H160,
    },
    /// Create scheme of `CREATE2`.
    Create2 {
        /// Caller of the create.
        caller: H160,
        /// Code hash.
        code_hash: H256,
        /// Salt.
        salt: H256,
    },
    /// Create at a fixed location.
    Fixed(H160),
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
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

/// Context of the runtime.
#[derive(Clone, Debug)]
pub struct Context {
    /// Execution address.
    pub address: H160,
    /// Caller of the EVM.
    pub caller: H160,
    /// Apparent value of the EVM.
    pub apparent_value: U256,
}


#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct GlobalContext {
    /// amount of gas that we can spend.
    pub gas_limit: U256,
	/// Gas price
	pub gas_price: U256,
    /// Get environmental block number.
    pub block_number: U256,
    /// Get environmental coinbase.
    pub block_coinbase: H160,
    /// Get environmental block timestamp.
    pub block_timestamp: U256,
    /// Get environmental block difficulty.
    pub block_difficulty: U256,
    /// Get environmental gas limit.
    pub block_gas_limit: U256,
    /// Get environmental chain ID.
    pub chain_id: U256,
    /// Get execution origin
    pub origin: H160,

}

/// Transfer from source to target, with given value.
#[derive(Clone, Debug)]
pub struct Transfer {
    /// Source address.
    pub source: H160,
    /// Target address.
    pub target: H160,
    /// Transfer value.
    pub value: U256,
}


#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Log {
	pub address: H160,
	pub topics: Vec<H256>,
	pub data: Bytes,
}
