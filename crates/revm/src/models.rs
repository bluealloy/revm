use core::cmp::min;

use crate::{alloc::vec::Vec, SpecId};
use bytes::Bytes;
use primitive_types::{H160, H256, U256};
use sha3::{Digest, Keccak256};

pub const KECCAK_EMPTY: H256 = H256([
    0xc5, 0xd2, 0x46, 0x01, 0x86, 0xf7, 0x23, 0x3c, 0x92, 0x7e, 0x7d, 0xb2, 0xdc, 0xc7, 0x03, 0xc0,
    0xe5, 0x00, 0xb6, 0x53, 0xca, 0x82, 0x27, 0x3b, 0x7b, 0xfa, 0xd8, 0x04, 0x5d, 0x85, 0xa4, 0x70,
]);

/// AccountInfo account information.
#[derive(Clone, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AccountInfo {
    /// Account balance.
    pub balance: U256,
    /// Account nonce.
    pub nonce: u64,
    /// code hash,
    pub code_hash: H256,
    /// code: if None, `code_by_hash` will be used to fetch it if code needs to be loaded from
    /// inside of revm.
    #[cfg_attr(feature = "with-serde", serde(with = "serde_hex_bytes_opt"))]
    pub code: Option<Bytes>,
}

impl Default for AccountInfo {
    fn default() -> Self {
        Self {
            balance: U256::zero(),
            code_hash: KECCAK_EMPTY,
            code: Some(Bytes::new()),
            nonce: 0,
        }
    }
}

impl AccountInfo {
    pub fn new(balance: U256, nonce: u64, code: Bytes) -> Self {
        let code_hash = if code.is_empty() {
            KECCAK_EMPTY
        } else {
            H256::from_slice(Keccak256::digest(&code).as_slice())
        };
        Self {
            balance,
            nonce,
            code: Some(code),
            code_hash,
        }
    }

    pub fn is_empty(&self) -> bool {
        let code_empty = self.code_hash == KECCAK_EMPTY || self.code_hash.is_zero();
        self.balance.is_zero() && self.nonce == 0 && code_empty
    }

    pub fn exists(&self) -> bool {
        !self.is_empty()
    }

    pub fn from_balance(balance: U256) -> Self {
        AccountInfo {
            balance,
            ..Default::default()
        }
    }
}

/// Inputs for a call.
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CallInputs {
    /// The target of the call.
    pub contract: H160,
    /// The transfer, if any, in this call.
    pub transfer: Transfer,
    /// The call data of the call.
    #[cfg_attr(feature = "with-serde", serde(with = "serde_hex_bytes"))]
    pub input: Bytes,
    /// The gas limit of the call.
    pub gas_limit: u64,
    /// The context of the call.
    pub context: CallContext,
}

#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CreateInputs {
    pub caller: H160,
    pub scheme: CreateScheme,
    pub value: U256,
    #[cfg_attr(feature = "with-serde", serde(with = "serde_hex_bytes"))]
    pub init_code: Bytes,
    pub gas_limit: u64,
}

pub struct CreateData {}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TransactOut {
    None,
    #[cfg_attr(feature = "with-serde", serde(with = "serde_hex_bytes"))]
    Call(Bytes),
    Create(
        #[cfg_attr(feature = "with-serde", serde(with = "serde_hex_bytes"))] Bytes,
        Option<H160>,
    ),
}

/// Create scheme.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CreateScheme {
    /// Legacy create scheme of `CREATE`.
    Create,
    /// Create scheme of `CREATE2`.
    Create2 {
        /// Salt.
        salt: U256,
    },
}

/// Call schemes.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
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
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CallContext {
    /// Execution address.
    pub address: H160,
    /// Caller of the EVM.
    pub caller: H160,
    /// The address the contract code was loaded from, if any.
    pub code_address: H160,
    /// Apparent value of the EVM.
    pub apparent_value: U256,
    /// The scheme used for the call.
    pub scheme: CallScheme,
}

impl Default for CallContext {
    fn default() -> Self {
        CallContext {
            address: H160::default(),
            caller: H160::default(),
            code_address: H160::default(),
            apparent_value: U256::default(),
            scheme: CallScheme::Call,
        }
    }
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Env {
    pub cfg: CfgEnv,
    pub block: BlockEnv,
    pub tx: TxEnv,
}
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BlockEnv {
    pub number: U256,
    /// Coinbase or miner or address that created and signed the block.
    /// Address where we are going to send gas spend
    pub coinbase: H160,
    pub timestamp: U256,
    pub difficulty: U256,
    /// basefee is added in EIP1559 London upgrade
    pub basefee: U256,
    pub gas_limit: U256,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TxEnv {
    /// Caller or Author or tx signer
    pub caller: H160,
    pub gas_limit: u64,
    pub gas_price: U256,
    pub gas_priority_fee: Option<U256>,
    pub transact_to: TransactTo,
    pub value: U256,
    #[cfg_attr(feature = "with-serde", serde(with = "serde_hex_bytes"))]
    pub data: Bytes,
    pub chain_id: Option<u64>,
    pub nonce: Option<u64>,
    pub access_list: Vec<(H160, Vec<U256>)>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CfgEnv {
    pub chain_id: U256,
    pub spec_id: SpecId,
    /// If all precompiles have some balance we can skip initially fetching them from the database.
    /// This is is not really needed on mainnet, and defaults to false, but in most cases it is
    /// safe to be set to `true`, depending on the chain.
    pub perf_all_precompiles_have_balance: bool,
    /// A hard memory limit in bytes beyond which [Memory] cannot be resized.
    ///
    /// In cases where the gas limit may be extraordinarily high, it is recommended to set this to
    /// a sane value to prevent memory allocation panics. Defaults to `2^32 - 1` bytes per
    /// EIP-1985.
    #[cfg(feature = "memory_limit")]
    pub memory_limit: u64,
}

impl Default for CfgEnv {
    fn default() -> CfgEnv {
        CfgEnv {
            chain_id: 1.into(),
            spec_id: SpecId::LATEST,
            perf_all_precompiles_have_balance: false,
            #[cfg(feature = "memory_limit")]
            memory_limit: 2u64.pow(32) - 1,
        }
    }
}

impl Default for BlockEnv {
    fn default() -> BlockEnv {
        BlockEnv {
            gas_limit: U256::MAX,
            number: 0.into(),
            coinbase: H160::zero(),
            timestamp: U256::one(),
            difficulty: U256::zero(),
            basefee: U256::zero(),
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

/// Transfer from source to target, with given value.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Transfer {
    /// Source address.
    pub source: H160,
    /// Target address.
    pub target: H160,
    /// Transfer value.
    pub value: U256,
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Log {
    pub address: H160,
    pub topics: Vec<H256>,
    #[cfg_attr(feature = "with-serde", serde(with = "serde_hex_bytes"))]
    pub data: Bytes,
}

#[derive(Default)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SelfDestructResult {
    pub had_value: bool,
    pub exists: bool,
    pub is_cold: bool,
    pub previously_destroyed: bool,
}

/// Serde functions to serde as [bytes::Bytes] hex string
#[cfg(feature = "with-serde")]
mod serde_hex_bytes {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S, T>(x: T, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AsRef<[u8]>,
    {
        s.serialize_str(&format!("0x{}", hex::encode(x.as_ref())))
    }

    pub fn deserialize<'de, D>(d: D) -> Result<bytes::Bytes, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(d)?;
        if let Some(value) = value.strip_prefix("0x") {
            hex::decode(value)
        } else {
            hex::decode(&value)
        }
        .map(Into::into)
        .map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}
/// Serde functions to serde an Option [bytes::Bytes] hex string
#[cfg(feature = "with-serde")]
mod serde_hex_bytes_opt {
    use super::serde_hex_bytes;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<T, S>(value: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AsRef<[u8]>,
    {
        if let Some(value) = value {
            serde_hex_bytes::serialize(value, serializer)
        } else {
            serializer.serialize_none()
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<bytes::Bytes>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(transparent)]
        struct OptionalBytes(Option<DeserializeBytes>);

        struct DeserializeBytes(bytes::Bytes);

        impl<'de> Deserialize<'de> for DeserializeBytes {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                Ok(DeserializeBytes(serde_hex_bytes::deserialize(
                    deserializer,
                )?))
            }
        }

        let value = OptionalBytes::deserialize(deserializer)?;
        Ok(value.0.map(|b| b.0))
    }
}
