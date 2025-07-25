use alloc::vec::Vec;

use crate::{deserializer::deserialize_maybe_empty, TestAuthorization};
use revm::{
    context::TransactionType,
    context_interface::transaction::AccessList,
    primitives::{Address, Bytes, B256, U256},
};
use serde::{Deserialize, Serialize};

/// Transaction parts.
#[derive(Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionParts {
    /// Transaction type (0=Legacy, 1=EIP-2930, 2=EIP-1559, 3=EIP-4844, 4=EIP-7702)
    #[serde(rename = "type")]
    pub tx_type: Option<u8>,
    /// Transaction data/input (multiple variants for different test cases)
    pub data: Vec<Bytes>,
    /// Gas limit values (multiple variants for different test cases)
    pub gas_limit: Vec<U256>,
    /// Gas price (for legacy and EIP-2930 transactions)
    pub gas_price: Option<U256>,
    /// Transaction nonce
    pub nonce: U256,
    /// Private key for signing the transaction
    pub secret_key: B256,
    /// if sender is not present we need to derive it from secret key.
    #[serde(default)]
    pub sender: Option<Address>,
    /// Recipient address (None for contract creation)
    #[serde(default, deserialize_with = "deserialize_maybe_empty")]
    pub to: Option<Address>,
    /// Ether value to transfer (multiple variants for different test cases)
    pub value: Vec<U256>,
    /// Maximum fee per gas (EIP-1559 transactions)
    pub max_fee_per_gas: Option<U256>,
    /// Maximum priority fee per gas (EIP-1559 transactions)
    pub max_priority_fee_per_gas: Option<U256>,
    /// Initcodes for contract creation (EIP-7873)
    pub initcodes: Option<Vec<Bytes>>,
    /// Access lists for different test cases (EIP-2930)
    #[serde(default)]
    pub access_lists: Vec<Option<AccessList>>,
    /// Authorization list (EIP-7702)
    pub authorization_list: Option<Vec<TestAuthorization>>,
    /// Blob versioned hashes (EIP-4844)
    #[serde(default)]
    pub blob_versioned_hashes: Vec<B256>,
    /// Maximum fee per blob gas (EIP-4844)
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
        if let Some(tx_type) = self.tx_type {
            return Some(TransactionType::from(tx_type));
        }

        let mut tx_type = TransactionType::Legacy;

        // If it has access list it is EIP-2930 tx
        if let Some(access_list) = self.access_lists.get(access_list_index) {
            if access_list.is_some() {
                tx_type = TransactionType::Eip2930;
            }
        }

        // If there is max_fee it is EIP-1559 tx
        if self.max_fee_per_gas.is_some() {
            tx_type = TransactionType::Eip1559;
        }

        // If it has max_fee_per_blob_gas it is EIP-4844 tx
        if self.max_fee_per_blob_gas.is_some() {
            // target need to be present for EIP-4844 tx
            self.to?;
            return Some(TransactionType::Eip4844);
        }

        // And if it has authorization list it is EIP-7702 tx
        if self.authorization_list.is_some() {
            // Target need to be present for EIP-7702 tx
            self.to?;
            return Some(TransactionType::Eip7702);
        }

        Some(tx_type)
    }
}

/// Transaction part indices.
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TxPartIndices {
    /// Index into the data array
    pub data: usize,
    /// Index into the gas_limit array
    pub gas: usize,
    /// Index into the value array
    pub value: usize,
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn decode_tx_parts() {
        let tx = r#"{
            "nonce": "0x00",
            "maxPriorityFeePerGas": "0x00",
            "maxFeePerGas": "0x07",
            "gasLimit": [
                "0x0423ff"
            ],
            "to": "0x0000000000000000000000000000000000001000",
            "value": [
                "0x00"
            ],
            "data": [
                "0x"
            ],
            "accessLists": [
                [
                    {
                        "address": "0x6389e7f33ce3b1e94e4325ef02829cd12297ef71",
                        "storageKeys": [
                            "0x0000000000000000000000000000000000000000000000000000000000000000"
                        ]
                    }
                ]
            ],
            "authorizationList": [
                {
                    "chainId": "0x00",
                    "address": "0xa94f5374fce5edbc8e2a8697c15331677e6ebf0b",
                    "nonce": "0x00",
                    "v": "0x01",
                    "r": "0x5a8cac98fd240d8ef83c22db4a061ffa0facb1801245283cc05fc809d8b92837",
                    "s": "0x1c3162fe11d91bc24d4fa00fb19ca34531e0eacdf8142c804be44058d5b8244f",
                    "signer": "0x6389e7f33ce3b1e94e4325ef02829cd12297ef71"
                }
            ],
            "sender": "0x8a0a19589531694250d570040a0c4b74576919b8",
            "secretKey": "0x9e7645d0cfd9c3a04eb7a9db59a4eb7d359f2e75c9164a9d6b9a7d54e1b6a36f"
        }"#;

        let _: TransactionParts = serde_json::from_str(tx).unwrap();
    }
}
