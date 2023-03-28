use crate::{alloc::vec::Vec, SpecId, B160, B256, U256};
use bytes::Bytes;
use core::cmp::min;

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Env {
    pub cfg: CfgEnv,
    pub block: BlockEnv,
    pub tx: TxEnv,
}
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BlockEnv {
    pub number: U256,
    /// Coinbase or miner or address that created and signed the block.
    /// Address where we are going to send gas spend
    pub coinbase: B160,
    pub timestamp: U256,
    /// Difficulty is removed and not used after Paris (aka TheMerge). Value is replaced with prevrandao.
    pub difficulty: U256,
    /// Prevrandao is used after Paris (aka TheMerge) instead of the difficulty value.
    /// NOTE: prevrandao can be found in block in place of mix_hash.
    pub prevrandao: Option<B256>,
    /// basefee is added in EIP1559 London upgrade
    pub basefee: U256,
    pub gas_limit: U256,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TxEnv {
    /// Caller or Author or tx signer
    pub caller: B160,
    pub gas_limit: u64,
    pub gas_price: U256,
    pub gas_priority_fee: Option<U256>,
    pub transact_to: TransactTo,
    pub value: U256,
    #[cfg_attr(feature = "serde", serde(with = "crate::utilities::serde_hex_bytes"))]
    pub data: Bytes,
    pub chain_id: Option<u64>,
    pub nonce: Option<u64>,
    pub access_list: Vec<(B160, Vec<U256>)>,
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TransactTo {
    Call(B160),
    Create(CreateScheme),
}

impl TransactTo {
    pub fn create() -> Self {
        Self::Create(CreateScheme::Create)
    }
    pub fn is_create(&self) -> bool {
        matches!(self, Self::Create(_))
    }
}

/// Create scheme.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CreateScheme {
    /// Legacy create scheme of `CREATE`.
    Create,
    /// Create scheme of `CREATE2`.
    Create2 {
        /// Salt.
        salt: U256,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CfgEnv {
    pub chain_id: U256,
    pub spec_id: SpecId,
    /// If all precompiles have some balance we can skip initially fetching them from the database.
    /// This is is not really needed on mainnet, and defaults to false, but in most cases it is
    /// safe to be set to `true`, depending on the chain.
    pub perf_all_precompiles_have_balance: bool,
    /// Bytecode that is created with CREATE/CREATE2 is by default analysed and jumptable is created.
    /// This is very benefitial for testing and speeds up execution of that bytecode if called multiple times.
    ///
    /// Default: Analyse
    pub perf_analyse_created_bytecodes: AnalysisKind,
    /// If some it will effects EIP-170: Contract code size limit. Usefull to increase this because of tests.
    /// By default it is 0x6000 (~25kb).
    pub limit_contract_code_size: Option<usize>,
    /// A hard memory limit in bytes beyond which [Memory] cannot be resized.
    ///
    /// In cases where the gas limit may be extraordinarily high, it is recommended to set this to
    /// a sane value to prevent memory allocation panics. Defaults to `2^32 - 1` bytes per
    /// EIP-1985.
    #[cfg(feature = "memory_limit")]
    pub memory_limit: u64,
    /// Skip balance checks if true. Adds transaction cost to balance to ensure execution doesn't fail.
    #[cfg(feature = "optional_balance_check")]
    pub disable_balance_check: bool,
    /// There are use cases where it's allowed to provide a gas limit that's higher than a block's gas limit. To that
    /// end, you can disable the block gas limit validation.
    /// By default, it is set to `false`.
    #[cfg(feature = "optional_block_gas_limit")]
    pub disable_block_gas_limit: bool,
    /// EIP-3607 rejects transactions from senders with deployed code. In development, it can be desirable to simulate
    /// calls from contracts, which this setting allows.
    /// By default, it is set to `false`.
    #[cfg(feature = "optional_eip3607")]
    pub disable_eip3607: bool,
    /// Disables all gas refunds. This is useful when using chains that have gas refunds disabled e.g. Avalanche.
    /// Reasoning behind removing gas refunds can be found in EIP-3298.
    /// By default, it is set to `false`.
    #[cfg(feature = "optional_gas_refund")]
    pub disable_gas_refund: bool,
    /// Disables base fee checks for EIP-1559 transactions.
    /// This is useful for testing method calls with zero gas price.
    #[cfg(feature = "optional_no_base_fee")]
    pub disable_base_fee: bool,
}

#[derive(Clone, Default, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AnalysisKind {
    Raw,
    Check,
    #[default]
    Analyse,
}

impl Default for CfgEnv {
    fn default() -> CfgEnv {
        CfgEnv {
            chain_id: U256::from(1),
            spec_id: SpecId::LATEST,
            perf_all_precompiles_have_balance: false,
            perf_analyse_created_bytecodes: Default::default(),
            limit_contract_code_size: None,
            #[cfg(feature = "memory_limit")]
            memory_limit: 2u64.pow(32) - 1,
            #[cfg(feature = "optional_balance_check")]
            disable_balance_check: false,
            #[cfg(feature = "optional_block_gas_limit")]
            disable_block_gas_limit: false,
            #[cfg(feature = "optional_eip3607")]
            disable_eip3607: false,
            #[cfg(feature = "optional_gas_refund")]
            disable_gas_refund: false,
            #[cfg(feature = "optional_no_base_fee")]
            disable_base_fee: false,
        }
    }
}

impl Default for BlockEnv {
    fn default() -> BlockEnv {
        BlockEnv {
            gas_limit: U256::MAX,
            number: U256::ZERO,
            coinbase: B160::zero(),
            timestamp: U256::from(1),
            difficulty: U256::ZERO,
            prevrandao: Some(B256::zero()),
            basefee: U256::ZERO,
        }
    }
}

impl Default for TxEnv {
    fn default() -> TxEnv {
        TxEnv {
            caller: B160::zero(),
            gas_limit: u64::MAX,
            gas_price: U256::ZERO,
            gas_priority_fee: None,
            transact_to: TransactTo::Call(B160::zero()), //will do nothing
            value: U256::ZERO,
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
