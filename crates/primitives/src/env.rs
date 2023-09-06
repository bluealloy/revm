use crate::{
    alloc::vec::Vec, Account, EVMError, InvalidTransaction, Spec, SpecId, B160, B256, KECCAK_EMPTY,
    MAX_INITCODE_SIZE, U256,
};
use bytes::Bytes;
use core::cmp::{min, Ordering};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Env {
    pub cfg: CfgEnv,
    pub block: BlockEnv,
    pub tx: TxEnv,
}
#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq, Eq)]
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
#[non_exhaustive]
pub struct CfgEnv {
    pub chain_id: U256,
    pub spec_id: SpecId,
    /// Bytecode that is created with CREATE/CREATE2 is by default analysed and jumptable is created.
    /// This is very beneficial for testing and speeds up execution of that bytecode if called multiple times.
    ///
    /// Default: Analyse
    pub perf_analyse_created_bytecodes: AnalysisKind,
    /// If some it will effects EIP-170: Contract code size limit. Useful to increase this because of tests.
    /// By default it is 0x6000 (~25kb).
    pub limit_contract_code_size: Option<usize>,
    /// Disables the coinbase tip during the finalization of the transaction. This is useful for
    /// rollups that redirect the tip to the sequencer.
    pub disable_coinbase_tip: bool,
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

impl CfgEnv {
    #[cfg(feature = "optional_eip3607")]
    pub fn is_eip3607_disabled(&self) -> bool {
        self.disable_eip3607
    }

    #[cfg(not(feature = "optional_eip3607"))]
    pub fn is_eip3607_disabled(&self) -> bool {
        false
    }

    #[cfg(feature = "optional_balance_check")]
    pub fn is_balance_check_disabled(&self) -> bool {
        self.disable_balance_check
    }

    #[cfg(not(feature = "optional_balance_check"))]
    pub fn is_balance_check_disabled(&self) -> bool {
        false
    }

    #[cfg(feature = "optional_gas_refund")]
    pub fn is_gas_refund_disabled(&self) -> bool {
        self.disable_gas_refund
    }

    #[cfg(not(feature = "optional_gas_refund"))]
    pub fn is_gas_refund_disabled(&self) -> bool {
        false
    }

    #[cfg(feature = "optional_no_base_fee")]
    pub fn is_base_fee_check_disabled(&self) -> bool {
        self.disable_base_fee
    }

    #[cfg(not(feature = "optional_no_base_fee"))]
    pub fn is_base_fee_check_disabled(&self) -> bool {
        false
    }

    #[cfg(feature = "optional_block_gas_limit")]
    pub fn is_block_gas_limit_disabled(&self) -> bool {
        self.disable_block_gas_limit
    }

    #[cfg(not(feature = "optional_block_gas_limit"))]
    pub fn is_block_gas_limit_disabled(&self) -> bool {
        false
    }
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
            perf_analyse_created_bytecodes: Default::default(),
            limit_contract_code_size: None,
            disable_coinbase_tip: false,
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

    /// Validate ENV data of the block.
    ///
    /// It can be skip if you are sure that PREVRANDAO is set.
    #[inline]
    pub fn validate_block_env<SPEC: Spec, T>(&self) -> Result<(), EVMError<T>> {
        // Prevrandao is required for merge
        if SPEC::enabled(SpecId::MERGE) && self.block.prevrandao.is_none() {
            return Err(EVMError::PrevrandaoNotSet);
        }
        Ok(())
    }

    /// Validate transaction data that is set inside ENV and return error if something is wrong.
    ///
    /// Return initial spend gas (Gas needed to execute transaction).
    #[inline]
    pub fn validate_tx<SPEC: Spec>(&self) -> Result<(), InvalidTransaction> {
        let gas_limit = self.tx.gas_limit;
        let effective_gas_price = self.effective_gas_price();
        let is_create = self.tx.transact_to.is_create();

        // BASEFEE tx check
        if SPEC::enabled(SpecId::LONDON) {
            if let Some(priority_fee) = self.tx.gas_priority_fee {
                if priority_fee > self.tx.gas_price {
                    // or gas_max_fee for eip1559
                    return Err(InvalidTransaction::GasMaxFeeGreaterThanPriorityFee);
                }
            }
            let basefee = self.block.basefee;

            // check minimal cost against basefee
            if !self.cfg.is_base_fee_check_disabled() && effective_gas_price < basefee {
                return Err(InvalidTransaction::GasPriceLessThanBasefee);
            }
        }

        // Check if gas_limit is more than block_gas_limit
        if !self.cfg.is_block_gas_limit_disabled() && U256::from(gas_limit) > self.block.gas_limit {
            return Err(InvalidTransaction::CallerGasLimitMoreThanBlock);
        }

        // EIP-3860: Limit and meter initcode
        if SPEC::enabled(SpecId::SHANGHAI) && is_create {
            let max_initcode_size = self
                .cfg
                .limit_contract_code_size
                .map(|limit| limit.saturating_mul(2))
                .unwrap_or(MAX_INITCODE_SIZE);
            if self.tx.data.len() > max_initcode_size {
                return Err(InvalidTransaction::CreateInitcodeSizeLimit);
            }
        }

        // Check if the transaction's chain id is correct
        if let Some(tx_chain_id) = self.tx.chain_id {
            if U256::from(tx_chain_id) != self.cfg.chain_id {
                return Err(InvalidTransaction::InvalidChainId);
            }
        }

        // Check if access list is empty for transactions before BERLIN
        if !SPEC::enabled(SpecId::BERLIN) && !self.tx.access_list.is_empty() {
            return Err(InvalidTransaction::AccessListNotSupported);
        }

        Ok(())
    }

    /// Validate transaction against state.
    #[inline]
    pub fn validate_tx_against_state(&self, account: &Account) -> Result<(), InvalidTransaction> {
        // EIP-3607: Reject transactions from senders with deployed code
        // This EIP is introduced after london but there was no collision in past
        // so we can leave it enabled always
        if !self.cfg.is_eip3607_disabled() && account.info.code_hash != KECCAK_EMPTY {
            return Err(InvalidTransaction::RejectCallerWithCode);
        }

        // Check that the transaction's nonce is correct
        if let Some(tx) = self.tx.nonce {
            let state = account.info.nonce;
            match tx.cmp(&state) {
                Ordering::Greater => {
                    return Err(InvalidTransaction::NonceTooHigh { tx, state });
                }
                Ordering::Less => {
                    return Err(InvalidTransaction::NonceTooLow { tx, state });
                }
                _ => {}
            }
        }

        let balance_check = U256::from(self.tx.gas_limit)
            .checked_mul(self.tx.gas_price)
            .and_then(|gas_cost| gas_cost.checked_add(self.tx.value))
            .ok_or(InvalidTransaction::OverflowPaymentInTransaction)?;

        // Check if account has enough balance for gas_limit*gas_price and value transfer.
        // Transfer will be done inside `*_inner` functions.
        if !self.cfg.is_balance_check_disabled() && balance_check > account.info.balance {
            return Err(InvalidTransaction::LackOfFundForMaxFee {
                fee: self.tx.gas_limit,
                balance: account.info.balance,
            });
        }

        Ok(())
    }
}
