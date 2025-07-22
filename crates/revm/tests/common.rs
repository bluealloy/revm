//! Common test utilities used to compare execution results against testdata.
#![allow(dead_code)]

use revm::{
    context::result::ResultAndState,
    context_interface::result::{ExecutionResult, HaltReason, Output, SuccessReason},
    primitives::Bytes,
    state::EvmState,
};

// Re-export the compare_or_save_testdata function from the common test util crate
pub(crate) use revm_common_test_util::compare_or_save_testdata;

// Re-export the constant for testdata directory path
pub(crate) const TESTS_TESTDATA: &str = "tests/testdata";

/// Example showing how to migrate an existing test to use the testdata comparison.
///
/// This example consists of:
/// 1. The "original" test with standard assertions
/// 2. The migration approach - running assertions and saving testdata
/// 3. The final migrated test that only uses testdata comparison
#[test]
fn template_test() {
    // Create a minimal result and state
    let result = ResultAndState::new(
        ExecutionResult::<HaltReason>::Success {
            reason: SuccessReason::Stop,
            gas_used: 1000,
            gas_refunded: 0,
            logs: vec![],
            output: Output::Call(Bytes::from(vec![4, 5, 6])),
        },
        EvmState::default(),
    );

    // Simply use the testdata comparison utility
    // No assertions needed - full validation is done by comparing with testdata
    compare_or_save_testdata("template_test.json", result);
}