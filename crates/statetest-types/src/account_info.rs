use revm::primitives::{Bytes, StorageKeyMap, StorageValue, U256};
use serde::Deserialize;

use crate::deserializer::deserialize_str_as_u64;

/// Account information
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AccountInfo {
    /// Account balance in wei
    pub balance: U256,
    /// Account bytecode
    pub code: Bytes,
    /// Account nonce (transaction count)
    #[serde(deserialize_with = "deserialize_str_as_u64")]
    pub nonce: u64,
    /// Account storage (key-value pairs)
    pub storage: StorageKeyMap<StorageValue>,
}
