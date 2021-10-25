use core::cmp::min;

use crate::collection::vec::Vec;
use bytes::Bytes;
use primitive_types::{H160, H256, U256};

pub const KECCAK_EMPTY: H256 = H256([
    0xc5, 0xd2, 0x46, 0x01, 0x86, 0xf7, 0x23, 0x3c, 0x92, 0x7e, 0x7d, 0xb2, 0xdc, 0xc7, 0x03, 0xc0,
    0xe5, 0x00, 0xb6, 0x53, 0xca, 0x82, 0x27, 0x3b, 0x7b, 0xfa, 0xd8, 0x04, 0x5d, 0x85, 0xa4, 0x70,
]);

/// AccountInfo account information.
#[derive(Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "with-codec", derive(codec::Encode, codec::Decode))]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AccountInfo {
    /// Account balance.
    pub balance: U256,
    /// code hash,
    pub code_hash: H256,
    /// code
    pub code: Option<Bytes>,
    /// Account nonce.
    pub nonce: u64,
}

impl Default for AccountInfo {
    fn default() -> Self {
        Self {
            balance: U256::zero(),
            code_hash: KECCAK_EMPTY,
            code: None,
            nonce: 0,
        }
    }
}

impl AccountInfo {
    pub fn is_empty(&self) -> bool {
        let code_empty = self.code_hash == KECCAK_EMPTY || self.code_hash == H256::zero();
        self.balance == U256::zero() && self.nonce == 0 && code_empty
    }

    pub fn exists(&self) -> bool {
        !self.is_empty()
    }

    pub fn from_balance(balance: U256) -> Self {
        let mut def = Self::default();
        def.balance = balance;
        def
    }
}

pub enum TransactTo {
    Call(H160),
    Create(CreateScheme),
}

impl TransactTo {
    pub fn create() -> Self {
        Self::Create(CreateScheme::Create)
    }
}

#[derive(Debug)]
pub enum TransactOut {
    None,
    Call(Bytes),
    Create(Bytes, Option<H160>),
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
#[derive(Clone, Debug, Default)]
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
    /// Gas price or after London it is
    pub gas_max_fee: U256,
    /// max priority fee introduced in LONDON upgrade
    pub gas_priority_fee: Option<U256>,
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
    pub block_basefee: Option<U256>,
    /// Get execution origin
    pub origin: H160,
}

impl GlobalEnv {
    pub fn effective_gas_price(&self) -> U256 {
        if self.block_basefee.is_none() || self.gas_priority_fee.is_none() {
            self.gas_max_fee
        } else {
            min(
                self.gas_max_fee,
                self.block_basefee.unwrap() + self.gas_priority_fee.unwrap(),
            )
        }
    }
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

#[derive(Default)]
pub struct SelfDestructResult {
    pub had_value: bool,
    pub exists: bool,
    pub is_cold: bool,
    pub previously_destroyed: bool,
}
