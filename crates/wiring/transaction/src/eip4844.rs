use crate::eip1559::Eip1559CommonTxFields;
use primitives::{Address, B256};
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
}
