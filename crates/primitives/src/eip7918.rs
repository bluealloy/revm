//! Constants for EIP-7918: Blob base fee bounded by execution cost
//!

/// Minimum base fee for blobs, if price of the blob is less than this value, this value will be used.
pub const BLOB_BASE_COST: u64 = 2_u64.pow(14);
