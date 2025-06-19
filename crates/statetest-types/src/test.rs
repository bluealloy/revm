use revm::primitives::{Address, Bytes, HashMap, B256};
use serde::Deserialize;

use crate::{transaction::TxPartIndices, AccountInfo};

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
