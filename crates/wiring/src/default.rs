pub mod block;
pub mod cfg;
pub mod tx;

pub use cfg::{AnalysisKind, CfgEnv};
use transaction::{Eip4844Tx, TransactionType};
pub use tx::TxEnv;

use crate::block::blob::calc_blob_gasprice;
use crate::{Block, EvmWiring, Transaction};
use core::fmt::Debug;
use core::hash::Hash;
use primitives::{TxKind, U256};
use std::boxed::Box;

/// Subtype
pub type EnvWiring<EvmWiringT> =
    Env<<EvmWiringT as EvmWiring>::Block, <EvmWiringT as EvmWiring>::Transaction>;

#[derive(Clone, Debug, Default)]
/// EVM environment configuration.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Env<BLOCK, TX> {
    /// Configuration of the EVM itself.
    pub cfg: CfgEnv,
    /// Configuration of the block the transaction is in.
    pub block: BLOCK,
    /// Configuration of the transaction that is being executed.
    pub tx: TX,
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
