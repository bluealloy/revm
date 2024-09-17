use crate::block::{BlobExcessGasAndPrice, Block};
use primitives::{Address, B256, U256};

/// The block environment.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BlockEnv {
    /// The number of ancestor blocks of this block (block height).
    pub number: U256,
    /// Coinbase or miner or address that created and signed the block.
    ///
    /// This is the receiver address of all the gas spent in the block.
    pub coinbase: Address,

    /// The timestamp of the block in seconds since the UNIX epoch.
    pub timestamp: U256,
    /// The gas limit of the block.
    pub gas_limit: U256,
    /// The base fee per gas, added in the London upgrade with [EIP-1559].
    ///
    /// [EIP-1559]: https://eips.ethereum.org/EIPS/eip-1559
    pub basefee: U256,
    /// The difficulty of the block.
    ///
    /// Unused after the Paris (AKA the merge) upgrade, and replaced by `prevrandao`.
    pub difficulty: U256,
    /// The output of the randomness beacon provided by the beacon chain.
    ///
    /// Replaces `difficulty` after the Paris (AKA the merge) upgrade with [EIP-4399].
    ///
    /// NOTE: `prevrandao` can be found in a block in place of `mix_hash`.
    ///
    /// [EIP-4399]: https://eips.ethereum.org/EIPS/eip-4399
    pub prevrandao: Option<B256>,
    /// Excess blob gas and blob gasprice.
    /// See also [`crate::block::calc_excess_blob_gas`]
    /// and [`crate::block::blob::calc_blob_gasprice`].
    ///
    /// Incorporated as part of the Cancun upgrade via [EIP-4844].
    ///
    /// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
    pub blob_excess_gas_and_price: Option<BlobExcessGasAndPrice>,
}

impl BlockEnv {
    /// Takes `blob_excess_gas` saves it inside env
    /// and calculates `blob_fee` with [`BlobExcessGasAndPrice`].
    pub fn set_blob_excess_gas_and_price(&mut self, excess_blob_gas: u64) {
        self.blob_excess_gas_and_price = Some(BlobExcessGasAndPrice::new(excess_blob_gas));
    }
}

impl Block for BlockEnv {
    #[inline]
    fn number(&self) -> &U256 {
        &self.number
    }

    #[inline]
    fn coinbase(&self) -> &Address {
        &self.coinbase
    }

    #[inline]
    fn timestamp(&self) -> &U256 {
        &self.timestamp
    }

    #[inline]
    fn gas_limit(&self) -> &U256 {
        &self.gas_limit
    }

    #[inline]
    fn basefee(&self) -> &U256 {
        &self.basefee
    }

    #[inline]
    fn difficulty(&self) -> &U256 {
        &self.difficulty
    }

    #[inline]
    fn prevrandao(&self) -> Option<&B256> {
        self.prevrandao.as_ref()
    }

    #[inline]
    fn blob_excess_gas_and_price(&self) -> Option<&BlobExcessGasAndPrice> {
        self.blob_excess_gas_and_price.as_ref()
    }
}

impl Default for BlockEnv {
    fn default() -> Self {
        Self {
            number: U256::ZERO,
            coinbase: Address::ZERO,
            timestamp: U256::from(1),
            gas_limit: U256::MAX,
            basefee: U256::ZERO,
            difficulty: U256::ZERO,
            prevrandao: Some(B256::ZERO),
            blob_excess_gas_and_price: Some(BlobExcessGasAndPrice::new(0)),
        }
    }
}
