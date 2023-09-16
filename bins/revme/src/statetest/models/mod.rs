use bytes::Bytes;
use revm::primitives::{HashMap, B160, B256, U256};
use serde::Deserialize;
use std::collections::BTreeMap;

mod deserializer;
use deserializer::*;

mod spec;
pub use self::spec::SpecName;

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct TestSuite(pub BTreeMap<String, TestUnit>);

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestUnit {
    #[serde(rename = "_info")]
    pub info: serde_json::Value,

    pub env: Env,
    pub pre: HashMap<B160, AccountInfo>,
    pub post: BTreeMap<SpecName, Vec<Test>>,
    pub transaction: TransactionParts,
}

/// State test indexed state result deserialization.
#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Test {
    pub expect_exception: Option<String>,

    /// Indexes
    pub indexes: TxPartIndices,

    /// Post state hash
    pub hash: B256,
    /// Post state
    #[serde(default)]
    pub post_state: HashMap<B160, AccountInfo>,

    /// Logs root
    pub logs: B256,

    /// Tx bytes
    #[serde(default, deserialize_with = "deserialize_opt_str_as_bytes")]
    pub txbytes: Option<Bytes>,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TxPartIndices {
    pub data: usize,
    pub gas: usize,
    pub value: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AccountInfo {
    pub balance: U256,
    #[serde(deserialize_with = "deserialize_str_as_bytes")]
    pub code: Bytes,
    #[serde(deserialize_with = "deserialize_str_as_u64")]
    pub nonce: u64,
    pub storage: HashMap<U256, U256>,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Env {
    pub current_coinbase: B160,
    #[serde(default, deserialize_with = "deserialize_str_as_u256")]
    pub current_difficulty: U256,
    #[serde(deserialize_with = "deserialize_str_as_u256")]
    pub current_gas_limit: U256,
    #[serde(deserialize_with = "deserialize_str_as_u256")]
    pub current_number: U256,
    #[serde(deserialize_with = "deserialize_str_as_u256")]
    pub current_timestamp: U256,
    pub current_base_fee: Option<U256>,
    pub previous_hash: B256,

    pub current_random: Option<B256>,
    pub current_beacon_root: Option<B256>,
    pub current_withdrawals_root: Option<B256>,

    pub parent_blob_gas_used: Option<U256>,
    pub parent_excess_blob_gas: Option<U256>,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TransactionParts {
    #[serde(deserialize_with = "deserialize_vec_as_vec_bytes")]
    pub data: Vec<Bytes>,
    pub gas_limit: Vec<U256>,
    pub gas_price: Option<U256>,
    pub nonce: U256,
    pub secret_key: B256,
    pub sender: B160,
    #[serde(deserialize_with = "deserialize_maybe_empty")]
    pub to: Option<B160>,
    pub value: Vec<U256>,
    pub max_fee_per_gas: Option<U256>,
    pub max_priority_fee_per_gas: Option<U256>,

    #[serde(default)]
    pub access_lists: Vec<Option<AccessList>>,

    #[serde(default)]
    pub blob_versioned_hashes: Vec<B256>,
    pub max_fee_per_blob_gas: Option<U256>,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AccessListItem {
    pub address: B160,
    pub storage_keys: Vec<B256>,
}

pub type AccessList = Vec<AccessListItem>;

#[cfg(test)]
mod tests {

    use super::*;
    use revm::primitives::B160;
    use serde_json::Error;

    #[test]
    pub fn serialize_u256() -> Result<(), Error> {
        let json = r#"{"_item":"0x10"}"#;

        #[derive(Deserialize, Debug)]
        pub struct Test {
            _item: Option<U256>,
        }

        let out: Test = serde_json::from_str(json)?;
        println!("out:{out:?}");
        Ok(())
    }

    #[test]
    pub fn serialize_b160() -> Result<(), Error> {
        let json = r#"{"_item":"0x2adc25665018aa1fe0e6bc666dac8fc2697ff9ba"}"#;

        #[derive(Deserialize, Debug)]
        pub struct Test {
            _item: B160,
        }

        let out: Test = serde_json::from_str(json)?;
        println!("out:{out:?}");
        Ok(())
    }
}
