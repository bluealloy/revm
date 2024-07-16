mod deserializer;
mod eip7702;
mod spec;

use deserializer::*;
pub use eip7702::TxEip7702;
pub use spec::SpecName;

use revm::primitives::{AccessList, Address, AuthorizationList, Bytes, HashMap, B256, U256};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct TestSuite(pub BTreeMap<String, TestUnit>);

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestUnit {
    /// Test info is optional
    #[serde(default, rename = "_info")]
    pub info: Option<serde_json::Value>,

    pub env: Env,
    pub pre: HashMap<Address, AccountInfo>,
    pub post: BTreeMap<SpecName, Vec<Test>>,
    pub transaction: TransactionParts,
    #[serde(default)]
    pub out: Option<Bytes>,
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
    pub post_state: HashMap<Address, AccountInfo>,

    /// Logs root
    pub logs: B256,

    /// Tx bytes
    pub txbytes: Option<Bytes>,
}

impl Test {
    pub fn eip7702_authorization_list(
        &self,
    ) -> Result<Option<AuthorizationList>, alloy_rlp::Error> {
        let Some(txbytes) = self.txbytes.as_ref() else {
            return Ok(None);
        };

        if txbytes.first() == Some(&0x04) {
            let mut txbytes = &txbytes[1..];
            let tx = TxEip7702::decode(&mut txbytes)?;
            return Ok(Some(
                AuthorizationList::Signed(tx.authorization_list).into_recovered(),
            ));
        }

        Ok(None)
    }
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
    pub code: Bytes,
    #[serde(deserialize_with = "deserialize_str_as_u64")]
    pub nonce: u64,
    pub storage: HashMap<U256, U256>,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Env {
    pub current_coinbase: Address,
    pub current_difficulty: U256,
    pub current_gas_limit: U256,
    pub current_number: U256,
    pub current_timestamp: U256,
    pub current_base_fee: Option<U256>,
    pub previous_hash: Option<B256>,

    pub current_random: Option<B256>,
    pub current_beacon_root: Option<B256>,
    pub current_withdrawals_root: Option<B256>,

    pub parent_blob_gas_used: Option<U256>,
    pub parent_excess_blob_gas: Option<U256>,
    pub current_excess_blob_gas: Option<U256>,
}

#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionParts {
    pub data: Vec<Bytes>,
    pub gas_limit: Vec<U256>,
    pub gas_price: Option<U256>,
    pub nonce: U256,
    pub secret_key: B256,
    /// if sender is not present we need to derive it from secret key.
    #[serde(default)]
    pub sender: Option<Address>,
    #[serde(deserialize_with = "deserialize_maybe_empty")]
    pub to: Option<Address>,
    pub value: Vec<U256>,
    pub max_fee_per_gas: Option<U256>,
    pub max_priority_fee_per_gas: Option<U256>,

    #[serde(default)]
    pub access_lists: Vec<Option<AccessList>>,

    //#[serde(default)]
    // TODO EIP-7702 when added enable serde `deny_unknown_fields`.
    //pub authorization_list: Vec<Option<Vec<TestAuthorization>>>,
    #[serde(default)]
    pub blob_versioned_hashes: Vec<B256>,
    pub max_fee_per_blob_gas: Option<U256>,
}

#[cfg(test)]
mod tests {

    use super::*;
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
    pub fn deserialize_minimal_transaction_parts() -> Result<(), Error> {
        let json = r#"{"data":[],"gasLimit":[],"nonce":"0x0","secretKey":"0x0000000000000000000000000000000000000000000000000000000000000000","to":"","value":[]}"#;

        let _: TransactionParts = serde_json::from_str(json)?;
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
