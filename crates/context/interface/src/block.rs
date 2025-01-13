pub mod blob;

pub use blob::{calc_blob_gasprice, calc_excess_blob_gas, BlobExcessGasAndPrice};

use auto_impl::auto_impl;
use primitives::{Address, B256, U256};
use std::boxed::Box;

/// Trait for retrieving block information required for execution.
#[auto_impl(&, &mut, Box, Arc)]
pub trait Block {
    /// The number of ancestor blocks of this block (block height).
    fn number(&self) -> u64;

    /// Beneficiary (Coinbase, miner) is a address that have signed the block.
    ///
    /// This is the receiver address of priority gas rewards.
    fn beneficiary(&self) -> Address;

    /// The timestamp of the block in seconds since the UNIX epoch.
    fn timestamp(&self) -> u64;

    /// The gas limit of the block.
    fn gas_limit(&self) -> u64;

    /// The base fee per gas, added in the London upgrade with [EIP-1559].
    ///
    /// [EIP-1559]: https://eips.ethereum.org/EIPS/eip-1559
    fn basefee(&self) -> u64;

    /// The difficulty of the block.
    ///
    /// Unused after the Paris (AKA the merge) upgrade, and replaced by `prevrandao`.
    fn difficulty(&self) -> U256;

    /// The output of the randomness beacon provided by the beacon chain.
    ///
    /// Replaces `difficulty` after the Paris (AKA the merge) upgrade with [EIP-4399].
    ///
    /// Note: `prevrandao` can be found in a block in place of `mix_hash`.
    ///
    /// [EIP-4399]: https://eips.ethereum.org/EIPS/eip-4399
    fn prevrandao(&self) -> Option<B256>;

    /// Excess blob gas and blob gasprice.
    /// See also [`calc_excess_blob_gas`]
    /// and [`calc_blob_gasprice`].
    ///
    /// Incorporated as part of the Cancun upgrade via [EIP-4844].
    ///
    /// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
    fn blob_excess_gas_and_price(&self) -> Option<BlobExcessGasAndPrice>;

    /// See [EIP-4844] and [`calc_blob_gasprice`].
    ///
    /// Returns `None` if `Cancun` is not enabled.
    ///
    /// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
    fn blob_gasprice(&self) -> Option<u128> {
        self.blob_excess_gas_and_price().map(|a| a.blob_gasprice)
    }

    /// Return `blob_excess_gas` header field. See [EIP-4844].
    ///
    /// Returns `None` if `Cancun` is not enabled.
    ///
    /// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
    fn blob_excess_gas(&self) -> Option<u64> {
        self.blob_excess_gas_and_price().map(|a| a.excess_blob_gas)
    }
}

#[auto_impl(&, &mut, Box, Arc)]
pub trait BlockGetter {
    type Block: Block;

    fn block(&self) -> &Self::Block;
}

pub trait BlockSetter: BlockGetter {
    fn set_block(&mut self, block: <Self as BlockGetter>::Block);
}

impl<T: BlockSetter> BlockSetter for &mut T {
    fn set_block(&mut self, block: <Self as BlockGetter>::Block) {
        (**self).set_block(block)
    }
}

impl<T: BlockSetter> BlockSetter for Box<T> {
    fn set_block(&mut self, block: <Self as BlockGetter>::Block) {
        (**self).set_block(block)
    }
}
