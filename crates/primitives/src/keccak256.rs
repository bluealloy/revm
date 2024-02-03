use alloy_primitives::B256;
use fluentbase_sdk::{LowLevelAPI, LowLevelSDK};

pub fn keccak256(input: &[u8]) -> B256 {
    let mut result = B256::ZERO;
    LowLevelSDK::crypto_keccak256(input, result.as_mut_slice());
    result
}
