use core::cmp::min;

use crate::{collection::vec::Vec, SpecId};
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

pub struct Env {
    pub cfg: CfgEnv,
    pub block: BlockEnv,
    pub tx: TxEnv,
}

pub struct BlockEnv {
    pub gas_limit: U256,
    /// somebody call it nonce
    pub number: U256,
    /// Coinbase or miner or address that created and signed the block.
    /// Address where we are going to send gas spend
    pub coinbase: H160,
    pub timestamp: U256,
    pub difficulty: U256,
    /// basefee is added in EIP1559 London upgrade
    pub basefee: U256,
    /// incrementaly added on every transaction. It can be cleared if needed
    pub gas_used: U256,
}

pub struct TxEnv {
    /// Caller or Author or tx signer
    pub caller: H160,
    pub gas_limit: u64,
    pub gas_price: U256,
    pub gas_priority_fee: Option<U256>,
    pub transact_to: TransactTo,
    pub value: U256,
    pub data: Bytes,
    pub chain_id: Option<u64>,
    pub nonce: Option<u64>,
    pub access_list: Vec<(H160, Vec<H256>)>,
}

pub struct CfgEnv {
    pub chain_id: U256,
    pub spec_id: SpecId,
}

impl Default for CfgEnv {
    fn default() -> CfgEnv {
        CfgEnv {
            chain_id: 1.into(), //mainnet is 1
            spec_id: SpecId::LATEST,
        }
    }
}

impl Default for BlockEnv {
    fn default() -> BlockEnv {
        BlockEnv {
            gas_limit: U256::MAX,
            number: 0.into(),
            coinbase: H160::zero(), //zero address
            timestamp: U256::one(),
            difficulty: U256::zero(),
            basefee: U256::zero(),
            gas_used: U256::zero(),
        }
    }
}

impl Default for TxEnv {
    fn default() -> TxEnv {
        TxEnv {
            caller: H160::zero(),
            gas_limit: u64::MAX,
            gas_price: U256::zero(),
            gas_priority_fee: None,
            transact_to: TransactTo::Call(H160::zero()), //will do nothing
            value: U256::zero(),
            data: Bytes::new(),
            chain_id: None,
            nonce: None,
            access_list: Vec::new(),
        }
    }
}

impl Env {
    pub fn effective_gas_price(&self) -> U256 {
        if self.tx.gas_priority_fee.is_none() {
            self.tx.gas_price
        } else {
            min(
                self.tx.gas_price,
                self.block.basefee + self.tx.gas_priority_fee.unwrap(),
            )
        }
    }
}

impl Default for Env {
    fn default() -> Env {
        Env {
            cfg: CfgEnv::default(),
            block: BlockEnv::default(),
            tx: TxEnv::default(),
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
