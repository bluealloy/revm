//! This contains a struct, [`RollupCostData`], that is used to compute the data availability costs
//! for a transaction.

use crate::optimism::fast_lz::flz_compress_len;
use revm_interpreter::gas::count_zero_bytes;

/// RollupCostData contains three fields, which are used depending on the current optimism fork.
///
/// The `zeroes` and `ones` fields are used to compute the data availability costs for a
/// transaction pre-fjord.
///
/// The `fastlz_size` field is used to compute the data availability costs for a transaction
/// post-fjord.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RollupCostData {
    /// The number of zeroes in the transaction.
    pub(crate) zeroes: u64,
    /// The number of ones in the transaction.
    pub(crate) ones: u64,
    /// The size of the transaction after fastLZ compression.
    pub(crate) fastlz_size: u32,
}

impl RollupCostData {
    /// This takes bytes as input, creating a [`RollupCostData`] struct based on the encoded data.
    pub fn from_bytes(bytes: &[u8]) -> Self {
        let (zeroes, ones) = count_zero_bytes(bytes);
        Self {
            zeroes,
            ones,
            fastlz_size: flz_compress_len(bytes),
        }
    }
}
