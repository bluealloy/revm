use serde::Deserialize;
use std::collections::BTreeMap;

use crate::TestUnit;

/// The top level test suite.
#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct TestSuite(pub BTreeMap<String, TestUnit>);
