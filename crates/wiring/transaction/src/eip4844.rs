use crate::eip1559::Eip1559CommonTxFields;
use primitives::B256;

pub trait Eip4844Tx: Eip1559CommonTxFields {
    /// Returns vector of fixed size hash(32 bytes)
    fn blob_versioned_hashes(&self) -> impl Iterator<Item = B256>;

    /// Max fee per data gas
    fn max_fee_per_blob_gas(&self) -> u128;
}
