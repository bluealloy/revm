use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};

use crate::{AccountInfo, Env, SpecName, Test, TransactionParts};
use revm::primitives::{Address, Bytes};

/// Single test unit struct
#[derive(Debug, PartialEq, Eq, Deserialize)]
//#[serde(deny_unknown_fields)]
// field config
pub struct TestUnit {
    /// Test info is optional.
    #[serde(default, rename = "_info")]
    pub info: Option<serde_json::Value>,

    pub env: Env,
    pub pre: HashMap<Address, AccountInfo>,
    pub post: BTreeMap<SpecName, Vec<Test>>,
    pub transaction: TransactionParts,
    #[serde(default)]
    pub out: Option<Bytes>,
    //pub config
}
