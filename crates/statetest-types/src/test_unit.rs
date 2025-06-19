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

    /// Test environment configuration.
    ///
    /// Contains the environmental information for executing the test, including
    /// block information, coinbase address, difficulty, gas limit, and other
    /// blockchain state parameters required for proper test execution.
    pub env: Env,

    /// Pre-execution state.
    ///
    /// A mapping of addresses to their account information before the transaction
    /// is executed. This represents the initial state of all accounts involved
    /// in the test, including their balances, nonces, code, and storage.
    pub pre: HashMap<Address, AccountInfo>,

    /// Post-execution expectations per specification.
    ///
    /// Maps each Ethereum specification name (hardfork) to a vector of expected
    /// test results. This allows a single test to define different expected outcomes
    /// for different protocol versions, enabling comprehensive testing across
    /// multiple Ethereum upgrades.
    pub post: BTreeMap<SpecName, Vec<Test>>,

    /// Transaction details to be executed.
    ///
    /// Contains the transaction parameters that will be executed against the
    /// pre-state. This includes sender, recipient, value, data, gas limits,
    /// and other transaction fields that may vary based on indices.
    pub transaction: TransactionParts,

    /// Expected output data from the transaction execution.
    ///
    /// Optional field containing the expected return data from the transaction.
    /// This is typically used for testing contract calls that return specific
    /// values or for CREATE operations that return deployed contract addresses.
    #[serde(default)]
    pub out: Option<Bytes>,
    //pub config
}
