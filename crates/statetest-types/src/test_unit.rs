use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};

use crate::{AccountInfo, Env, SpecName, Test, TransactionParts};
use revm::primitives::{Address, Bytes};

/// A single test unit.
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
