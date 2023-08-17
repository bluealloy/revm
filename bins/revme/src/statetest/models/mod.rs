use revm::primitives::{Address, Bytes, HashMap, B256, U256};
use serde::Deserialize;
use std::collections::BTreeMap;

mod deserializer;
use deserializer::*;

mod spec;
pub use self::spec::SpecName;

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct TestSuite(pub BTreeMap<String, TestUnit>);

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct TestUnit {
    pub env: Env,
    pub pre: HashMap<Address, AccountInfo>,
    pub post: BTreeMap<SpecName, Vec<Test>>,
    pub transaction: TransactionParts,
}

/// State test indexed state result deserialization.
#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct Test {
    /// Post state hash
    pub hash: B256,
    /// Indexes
    pub indexes: TxPartIndices,
    /// Logs
    pub logs: B256,
    /// Transaction bytes
    #[serde(default)]
    pub txbytes: Option<Bytes>,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct TxPartIndices {
    pub data: usize,
    pub gas: usize,
    pub value: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    pub balance: U256,
    pub code: Bytes,
    #[serde(deserialize_with = "deserialize_str_as_u64")]
    pub nonce: u64,
    pub storage: HashMap<U256, U256>,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Env {
    pub current_coinbase: Address,
    pub current_difficulty: U256,
    pub current_gas_limit: U256,
    pub current_number: U256,
    pub current_timestamp: U256,
    pub current_base_fee: Option<U256>,
    pub previous_hash: B256,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionParts {
    pub data: Vec<Bytes>,
    pub access_lists: Option<Vec<Option<AccessList>>>,
    pub gas_limit: Vec<U256>,
    pub gas_price: Option<U256>,
    pub nonce: U256,
    pub secret_key: Option<B256>,
    #[serde(deserialize_with = "deserialize_maybe_empty")]
    pub to: Option<Address>,
    pub value: Vec<U256>,
    pub max_fee_per_gas: Option<U256>,
    pub max_priority_fee_per_gas: Option<U256>,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccessListItem {
    pub address: Address,
    pub storage_keys: Vec<B256>,
}

pub type AccessList = Vec<AccessListItem>;

#[cfg(test)]
mod tests {

    use super::*;
    use revm::primitives::Address;
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
            _item: Address,
        }

        let out: Test = serde_json::from_str(json)?;
        println!("out:{out:?}");
        Ok(())
    }
}
