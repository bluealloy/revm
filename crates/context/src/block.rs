//! This module contains [`BlockEnv`] and it implements [`Block`] trait.
use context_interface::block::{BlobExcessGasAndPrice, Block};
use primitives::{Address, B256, U256};

/// The block environment
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BlockEnv {
    /// The number of ancestor blocks of this block (block height)
    pub number: u64,
    /// Beneficiary (Coinbase or miner) is a address that have signed the block
    ///
    /// This is the receiver address of all the gas spent in the block.
    pub beneficiary: Address,

    /// The timestamp of the block in seconds since the UNIX epoch
    pub timestamp: u64,
    /// The gas limit of the block
    pub gas_limit: u64,
    /// The base fee per gas, added in the London upgrade with [EIP-1559]
    ///
    /// [EIP-1559]: https://eips.ethereum.org/EIPS/eip-1559
    pub basefee: u64,
    /// The difficulty of the block
    ///
    /// Unused after the Paris (AKA the merge) upgrade, and replaced by `prevrandao`.
    pub difficulty: U256,
    /// The output of the randomness beacon provided by the beacon chain
    ///
    /// Replaces `difficulty` after the Paris (AKA the merge) upgrade with [EIP-4399].
    ///
    /// Note: `prevrandao` can be found in a block in place of `mix_hash`.
    ///
    /// [EIP-4399]: https://eips.ethereum.org/EIPS/eip-4399
    pub prevrandao: Option<B256>,
    /// Excess blob gas and blob gasprice
    ///
    /// See also [`calc_excess_blob_gas`][context_interface::block::calc_excess_blob_gas]
    /// and [`calc_blob_gasprice`][context_interface::block::blob::calc_blob_gasprice].
    ///
    /// Incorporated as part of the Cancun upgrade via [EIP-4844].
    ///
    /// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
    pub blob_excess_gas_and_price: Option<BlobExcessGasAndPrice>,
}

impl BlockEnv {
    /// Takes `blob_excess_gas` saves it inside env
    /// and calculates `blob_fee` with [`BlobExcessGasAndPrice`].
    pub fn set_blob_excess_gas_and_price(&mut self, excess_blob_gas: u64, is_prague: bool) {
        self.blob_excess_gas_and_price =
            Some(BlobExcessGasAndPrice::new(excess_blob_gas, is_prague));
    }
}

impl Block for BlockEnv {
    #[inline]
    fn number(&self) -> u64 {
        self.number
    }

    #[inline]
    fn beneficiary(&self) -> Address {
        self.beneficiary
    }

    #[inline]
    fn timestamp(&self) -> u64 {
        self.timestamp
    }

    #[inline]
    fn gas_limit(&self) -> u64 {
        self.gas_limit
    }

    #[inline]
    fn basefee(&self) -> u64 {
        self.basefee
    }

    #[inline]
    fn difficulty(&self) -> U256 {
        self.difficulty
    }

    #[inline]
    fn prevrandao(&self) -> Option<B256> {
        self.prevrandao
    }

    #[inline]
    fn blob_excess_gas_and_price(&self) -> Option<BlobExcessGasAndPrice> {
        self.blob_excess_gas_and_price
    }
}

impl Default for BlockEnv {
    fn default() -> Self {
        Self {
            number: 0,
            beneficiary: Address::ZERO,
            timestamp: 1,
            gas_limit: u64::MAX,
            basefee: 0,
            difficulty: U256::ZERO,
            prevrandao: Some(B256::ZERO),
            blob_excess_gas_and_price: Some(BlobExcessGasAndPrice::new(0, false)),
        }
    }
}
