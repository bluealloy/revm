use revm::{
    primitives::{Address, Bytes, B256, U256},
    specification::eip2930::AccessList,
    transaction::TransactionType,
};
use serde::{Deserialize, Serialize};

use crate::{deserializer::deserialize_maybe_empty, TestAuthorization};

/// Transaction parts.
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
    #[serde(default, deserialize_with = "deserialize_maybe_empty")]
    pub to: Option<Address>,
    pub value: Vec<U256>,
    pub max_fee_per_gas: Option<U256>,
    pub max_priority_fee_per_gas: Option<U256>,

    #[serde(default)]
    pub access_lists: Vec<Option<AccessList>>,
    pub authorization_list: Option<Vec<TestAuthorization>>,
    #[serde(default)]
    pub blob_versioned_hashes: Vec<B256>,
    pub max_fee_per_blob_gas: Option<U256>,
}

impl TransactionParts {
    /// Returns the transaction type.   
    ///
    /// As this information is derived from the fields it is not stored in the struct.
    ///
    /// Returns `None` if the transaction is invalid:
    ///   * It has both blob gas and no destination.
    ///   * It has authorization list and no destination.
    pub fn tx_type(&self, access_list_index: usize) -> Option<TransactionType> {
        let mut tx_type = TransactionType::Legacy;

        // if it has access list it is EIP-2930 tx
        if let Some(access_list) = self.access_lists.get(access_list_index) {
            if access_list.is_some() {
                tx_type = TransactionType::Eip2930;
            }
        }

        // If there is max_fee it is EIP-1559 tx
        if self.max_fee_per_gas.is_some() {
            tx_type = TransactionType::Eip1559;
        }

        // if it has max_fee_per_blob_gas it is EIP-4844 tx
        if self.max_fee_per_blob_gas.is_some() {
            // target need to be present for EIP-4844 tx
            self.to?;
            tx_type = TransactionType::Eip4844;
        }

        // and if it has authorization list it is EIP-7702 tx
        if self.authorization_list.is_some() {
            // target need to be present for EIP-7702 tx
            self.to?;
            tx_type = TransactionType::Eip7702;
        }

        Some(tx_type)
    }
}

/// Transaction part indices.
#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TxPartIndices {
    pub data: usize,
    pub gas: usize,
    pub value: usize,
}
