pub mod blob;

pub use blob::{calc_blob_gasprice, calc_excess_blob_gas, BlobExcessGasAndPrice};

use primitives::{Address, B256, U256};

/// Trait for retrieving block information required for execution.
pub trait Block {
    /// The number of ancestor blocks of this block (block height).
    fn number(&self) -> &U256;

    /// Coinbase or miner or address that created and signed the block.
    ///
    /// This is the receiver address of all the gas spent in the block.
    fn coinbase(&self) -> &Address;

    /// The timestamp of the block in seconds since the UNIX epoch.
    fn timestamp(&self) -> &U256;

    /// The gas limit of the block.
    fn gas_limit(&self) -> &U256;

    /// The base fee per gas, added in the London upgrade with [EIP-1559].
    ///
    /// [EIP-1559]: https://eips.ethereum.org/EIPS/eip-1559
    fn basefee(&self) -> &U256;

    /// The difficulty of the block.
    ///
    /// Unused after the Paris (AKA the merge) upgrade, and replaced by `prevrandao`.
    fn difficulty(&self) -> &U256;

    /// The output of the randomness beacon provided by the beacon chain.
    ///
    /// Replaces `difficulty` after the Paris (AKA the merge) upgrade with [EIP-4399].
    ///
    /// NOTE: `prevrandao` can be found in a block in place of `mix_hash`.
    ///
    /// [EIP-4399]: https://eips.ethereum.org/EIPS/eip-4399
    fn prevrandao(&self) -> Option<&B256>;

    /// Excess blob gas and blob gasprice.
    /// See also [`calc_excess_blob_gas`]
    /// and [`calc_blob_gasprice`].
    ///
    /// Incorporated as part of the Cancun upgrade via [EIP-4844].
    ///
    /// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
    fn blob_excess_gas_and_price(&self) -> Option<&BlobExcessGasAndPrice>;

    /// See [EIP-4844] and [`calc_blob_gasprice`].
    ///
    /// Returns `None` if `Cancun` is not enabled. This is enforced in [`crate::default::Env::validate_block_env`].
    ///
    /// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
    fn get_blob_gasprice(&self) -> Option<&u128> {
        self.blob_excess_gas_and_price().map(|a| &a.blob_gasprice)
    }

    /// Return `blob_excess_gas` header field. See [EIP-4844].
    ///
    /// Returns `None` if `Cancun` is not enabled. This is enforced in [`crate::default::Env::validate_block_env`].
    ///
    /// [EIP-4844]: https://eips.ethereum.org/EIPS/eip-4844
    fn get_blob_excess_gas(&self) -> Option<u64> {
        self.blob_excess_gas_and_price().map(|a| a.excess_blob_gas)
    }
}
