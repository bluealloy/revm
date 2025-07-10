use alloc::collections::BTreeMap;
use alloc::string::String;

use serde::Deserialize;

use crate::TestUnit;

/// The top level test suite struct
#[derive(Debug, PartialEq, Eq, Deserialize)]
pub struct TestSuite(pub BTreeMap<String, TestUnit>);
