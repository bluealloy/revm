use crate::eip1559::Eip1559CommonTxFields;
use primitives::{Address, B256, U256};
use specification::eip4844::GAS_PER_BLOB;

pub trait Eip4844Tx: Eip1559CommonTxFields {
    /// Call destination
    fn destination(&self) -> Address;

    /// Returns vector of fixed size hash(32 bytes)
    fn blob_versioned_hashes(&self) -> &[B256];

    /// Max fee per data gas
    fn max_fee_per_blob_gas(&self) -> u128;

    /// Total gas for all blobs. Max number of blocks is already checked
    /// so we dont need to check for overflow.
    fn total_blob_gas(&self) -> u64 {
        GAS_PER_BLOB * self.blob_versioned_hashes().len() as u64
    }

    /// Calculates the maximum [EIP-4844] `data_fee` of the transaction.
    ///
    /// This is used for ensuring that the user has at least enough funds to pay the
    /// `max_fee_per_blob_gas * total_blob_gas`, on top of regular gas costs.
    ///
    /// See EIP-4844:
    /// <https://github.com/ethereum/EIPs/blob/master/EIPS/eip-4844.md#execution-layer-validation>
    fn calc_max_data_fee(&self) -> U256 {
        let blob_gas = U256::from(self.total_blob_gas());
        let max_blob_fee = U256::from(self.max_fee_per_blob_gas());
        max_blob_fee.saturating_mul(blob_gas)
    }
}
