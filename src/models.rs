use crate::collection::vec::Vec;
use bytes::Bytes;
use primitive_types::{H160, H256, U256};

/// AccountInfo account information.
#[derive(Clone, Eq, PartialEq, Debug, Default)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AccountInfo {
    /// Account balance.
    pub balance: U256,
    /// code hash,
    pub code_hash: Option<H256>,
    /// code
    pub code: Option<Bytes>,
    /// Account nonce.
    pub nonce: u64,
}

impl AccountInfo {
    pub fn is_empty(&self) -> bool {
        self.nonce == 0
            && self.balance == U256::zero()
            && (self.code_hash == None || self.code_hash == Some(H256::zero()))
    }

    pub fn exists(&self) -> bool {
        !self.is_empty()
    }
}

/// Create scheme.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum CreateScheme {
    /// Legacy create scheme of `CREATE`.
    Create,
    /// Create scheme of `CREATE2`.
    Create2 {
        /// Salt.
        salt: H256,
    },
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

/// CallContext of the runtime.
#[derive(Clone, Debug)]
pub struct CallContext {
    /// Execution address.
    pub address: H160,
    /// Caller of the EVM.
    pub caller: H160,
    /// Apparent value of the EVM.
    pub apparent_value: U256,
}

#[derive(Clone, Eq, PartialEq, Debug, Default)]
pub struct GlobalEnv {
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
