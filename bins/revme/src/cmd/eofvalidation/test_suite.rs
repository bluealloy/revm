use revm::primitives::Bytes;
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct TestSuite(pub BTreeMap<String, TestUnit>);

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TestUnit {
    #[serde(default, rename = "_info")]
    pub info: Option<serde_json::Value>,
    #[serde(default)]
    pub vectors: BTreeMap<String, TestVector>,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TestVector {
    pub code: Bytes,
    pub container_kind: Option<String>,
    pub results: PragueTestResult,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PragueTestResult {
    #[serde(rename = "Prague")]
    pub prague: TestResult,
}

#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct TestResult {
    pub result: bool,
    pub exception: Option<String>,
}
