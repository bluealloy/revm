use revm::{
    context::tx::TxEnv,
    primitives::{Address, Bytes, HashMap, TxKind, B256},
};
use serde::Deserialize;

use crate::{
    error::TestError, transaction::TxPartIndices, utils::recover_address, AccountInfo, TestUnit,
};

/// State test indexed state result deserialization.
#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Test {
    /// Expected exception for this test case, if any.
    ///
    /// This field contains an optional string describing an expected error or exception
    /// that should occur during the execution of this state test. If present, the test
    /// is expected to fail with this specific error message or exception type.
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

    /// Output state.
    ///
    /// Note: Not used.
    #[serde(default)]
    state: HashMap<Address, AccountInfo>,

    /// Tx bytes
    pub txbytes: Option<Bytes>,
}

impl Test {
    /// Create a transaction environment from this test and the test unit.
    ///
    /// This function sets up the transaction environment using the test's
    /// indices to select the appropriate transaction parameters from the
    /// test unit.
    ///
    /// # Arguments
    ///
    /// * `unit` - The test unit containing transaction parts
    ///
    /// # Returns
    ///
    /// A configured [`TxEnv`] ready for execution, or an error if setup fails
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The private key cannot be used to recover the sender address
    /// - The transaction type is invalid and no exception is expected
    pub fn tx_env(&self, unit: &TestUnit) -> Result<TxEnv, TestError> {
        // Setup sender
        let caller = if let Some(address) = unit.transaction.sender {
            address
        } else {
            recover_address(unit.transaction.secret_key.as_slice())
                .ok_or(TestError::UnknownPrivateKey(unit.transaction.secret_key))?
        };

        // Transaction specific fields
        let tx_type = unit.transaction.tx_type(self.indexes.data).ok_or_else(|| {
            if self.expect_exception.is_some() {
                TestError::UnexpectedException {
                    expected_exception: self.expect_exception.clone(),
                    got_exception: Some("Invalid transaction type".to_string()),
                }
            } else {
                TestError::InvalidTransactionType
            }
        })?;

        let tx = TxEnv::builder()
            .caller(caller)
            .gas_price(unit
                .transaction
                .gas_price
                .or(unit.transaction.max_fee_per_gas)
                .unwrap_or_default()
                .try_into()
                .unwrap_or(u128::MAX))
            .gas_priority_fee(unit
                .transaction
                .max_priority_fee_per_gas
                .map(|b| u128::try_from(b).expect("max priority fee less than u128::MAX")))
            .blob_hashes(unit.transaction.blob_versioned_hashes.clone())
            .max_fee_per_blob_gas(unit
                .transaction
                .max_fee_per_blob_gas
                .map(|b| u128::try_from(b).expect("max fee less than u128::MAX"))
                .unwrap_or(u128::MAX))
            .tx_type(Some(tx_type as u8))
            .gas_limit(unit.transaction.gas_limit[self.indexes.gas].saturating_to())
            .data(unit.transaction.data[self.indexes.data].clone())
            .nonce(u64::try_from(unit.transaction.nonce).unwrap())
            .value(unit.transaction.value[self.indexes.value])
            .access_list(unit
                .transaction
                .access_lists
                .get(self.indexes.data)
                .cloned()
                .flatten()
                .unwrap_or_default())
            .authorization_list(unit
                .transaction
                .authorization_list
                .clone()
                .map(|auth_list| {
                    auth_list
                        .into_iter()
                        .map(|i| revm::context::either::Either::Left(i.into()))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default())
            .kind(match unit.transaction.to {
                Some(add) => TxKind::Call(add),
                None => TxKind::Create,
            })
            .build()
            .unwrap();

        Ok(tx)
    }
}
