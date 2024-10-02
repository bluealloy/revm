pub mod block;
pub mod tx;

use transaction::{Eip4844Tx, TransactionType};
pub use tx::TxEnv;

use crate::block::blob::calc_blob_gasprice;
use crate::{Block, EvmWiring, Transaction};
use core::fmt::Debug;
use core::hash::Hash;
use primitives::{TxKind, U256};
use specification::constants::MAX_CODE_SIZE;
use std::boxed::Box;

/// Subtype
pub type EnvWiring<EvmWiringT> =
    Env<<EvmWiringT as EvmWiring>::Block, <EvmWiringT as EvmWiring>::Transaction>;

#[derive(Clone, Debug, Default)]
/// EVM environment configuration.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Env<BlockT: Block, TxT: Transaction> {
    /// Configuration of the EVM itself.
    pub cfg: CfgEnv,
    /// Configuration of the block the transaction is in.
    pub block: BlockT,
    /// Configuration of the transaction that is being executed.
    pub tx: TxT,
}

impl<BlockT: Block, TxT: Transaction> Env<BlockT, TxT> {
    /// Create boxed [Env].
    #[inline]
    pub fn boxed(cfg: CfgEnv, block: BlockT, tx: TxT) -> Box<Self> {
        Box::new(Self { cfg, block, tx })
    }

    pub fn effective_gas_price(&self) -> U256 {
        let basefee = *self.block.basefee();
        self.tx.effective_gas_price(basefee)
    }

    /// Calculates the [EIP-4844] `data_fee` of the transaction.
    ///
    /// Returns `None` if `Cancun` is not enabled.
    ///
    /// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
    #[inline]
    pub fn calc_data_fee(&self) -> Option<U256> {
        if self.tx.tx_type().into() == TransactionType::Eip4844 {
            let blob_gas = U256::from(self.tx.eip4844().total_blob_gas());
            let blob_gas_price = U256::from(self.block.blob_gasprice().unwrap_or_default());
            return Some(blob_gas_price.saturating_mul(blob_gas));
        }
        None
    }

    /// Calculates the maximum [EIP-4844] `data_fee` of the transaction.
    ///
    /// This is used for ensuring that the user has at least enough funds to pay the
    /// `max_fee_per_blob_gas * total_blob_gas`, on top of regular gas costs.
    ///
    /// See EIP-4844:
    /// <https://github.com/ethereum/EIPs/blob/master/EIPS/eip-4844.md#execution-layer-validation>
    pub fn calc_max_data_fee(&self) -> Option<U256> {
        if self.tx.tx_type().into() == TransactionType::Eip4844 {
            let blob_gas = U256::from(self.tx.eip4844().total_blob_gas());
            let max_blob_fee = U256::from(self.tx.eip4844().max_fee_per_blob_gas());
            return Some(max_blob_fee.saturating_mul(blob_gas));
        }
        None
    }
}

impl<BlockT: Block + Default, TxT: Transaction + Default> Env<BlockT, TxT> {
    /// Resets environment to default values.
    #[inline]
    pub fn clear(&mut self) {
        *self = Self::default();
    }
}

/// EVM configuration.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct CfgEnv {
    /// Chain ID of the EVM, it will be compared to the transaction's Chain ID.
    /// Chain ID is introduced EIP-155
    pub chain_id: u64,
    /// KZG Settings for point evaluation precompile. By default, this is loaded from the ethereum mainnet trusted setup.
    #[cfg(any(feature = "c-kzg", feature = "kzg-rs"))]
    #[cfg_attr(feature = "serde", serde(skip))]
    pub kzg_settings: crate::kzg::EnvKzgSettings,
    /// Bytecode that is created with CREATE/CREATE2 is by default analysed and jumptable is created.
    /// This is very beneficial for testing and speeds up execution of that bytecode if called multiple times.
    ///
    /// Default: Analyse
    pub perf_analyse_created_bytecodes: AnalysisKind,
    /// If some it will effects EIP-170: Contract code size limit. Useful to increase this because of tests.
    /// By default it is 0x6000 (~25kb).
    pub limit_contract_code_size: Option<usize>,
    /// Skips the nonce validation against the account's nonce.
    pub disable_nonce_check: bool,
    /// A hard memory limit in bytes beyond which [crate::result::OutOfGasError::Memory] cannot be resized.
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
    /// By default, it is set to `false`.
    #[cfg(feature = "optional_no_base_fee")]
    pub disable_base_fee: bool,
}

impl CfgEnv {
    /// Returns max code size from [`Self::limit_contract_code_size`] if set
    /// or default [`MAX_CODE_SIZE`] value.
    pub fn max_code_size(&self) -> usize {
        self.limit_contract_code_size.unwrap_or(MAX_CODE_SIZE)
    }

    pub fn with_chain_id(mut self, chain_id: u64) -> Self {
        self.chain_id = chain_id;
        self
    }

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

    pub const fn is_nonce_check_disabled(&self) -> bool {
        self.disable_nonce_check
    }
}

impl Default for CfgEnv {
    fn default() -> Self {
        Self {
            chain_id: 1,
            perf_analyse_created_bytecodes: AnalysisKind::default(),
            limit_contract_code_size: None,
            disable_nonce_check: false,
            #[cfg(any(feature = "c-kzg", feature = "kzg-rs"))]
            kzg_settings: crate::kzg::EnvKzgSettings::Default,
            #[cfg(feature = "memory_limit")]
            memory_limit: (1 << 32) - 1,
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

/// Structure holding block blob excess gas and it calculates blob fee.
///
/// Incorporated as part of the Cancun upgrade via [EIP-4844].
///
/// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BlobExcessGasAndPrice {
    /// The excess blob gas of the block.
    pub excess_blob_gas: u64,
    /// The calculated blob gas price based on the `excess_blob_gas`, See [calc_blob_gasprice]
    pub blob_gasprice: u128,
}

impl BlobExcessGasAndPrice {
    /// Creates a new instance by calculating the blob gas price with [`calc_blob_gasprice`].
    pub fn new(excess_blob_gas: u64) -> Self {
        let blob_gasprice = calc_blob_gasprice(excess_blob_gas);
        Self {
            excess_blob_gas,
            blob_gasprice,
        }
    }
}

/// Transaction destination
pub type TransactTo = TxKind;

/// Create scheme.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
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

/// What bytecode analysis to perform.
#[derive(Clone, Default, Debug, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AnalysisKind {
    /// Do not perform bytecode analysis.
    Raw,
    /// Perform bytecode analysis.
    #[default]
    Analyse,
}

#[cfg(test)]
mod tests {
    // use super::*;
    // use crate::default::block::BlockEnv;
    // use specification::hardfork::{FrontierSpec, LatestSpec};

    // #[test]
    // fn test_validate_tx_chain_id() {
    //     let mut env = Env::<BlockEnv, TxEnv>::default();
    //     env.tx.chain_id = Some(1);
    //     env.cfg.chain_id = 2;
    //     assert_eq!(
    //         env.validate_tx::<LatestSpec>(),
    //         Err(InvalidTransaction::InvalidChainId)
    //     );
    // }

    // #[test]
    // fn test_validate_tx_access_list() {
    //     let mut env = Env::<BlockEnv, TxEnv>::default();
    //     env.tx.access_list = vec![AccessListItem {
    //         address: Address::ZERO,
    //         storage_keys: vec![],
    //     }]
    //     .into();
    //     assert_eq!(
    //         env.validate_tx::<FrontierSpec>(),
    //         Err(InvalidTransaction::AccessListNotSupported)
    //     );
    // }
}
