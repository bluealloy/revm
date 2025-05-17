mod test_utils;

use revm::{
    primitives::Bytes,
    context_interface::result::{ExecutionResult, HaltReason, ResultAndState, SuccessReason, Output},
    state::EvmState,
};

use crate::test_utils::{compare_or_save_testdata, migrate_test_to_testdata};

/// Example showing how to migrate an existing test to use the testdata comparison.
/// 
/// This example consists of:
/// 1. The "original" test with standard assertions
/// 2. The migration helper that wraps the original assertions
/// 3. The final migrated test that only uses testdata comparison
#[test]
fn migration_example() {
    // Create a minimal result and state to test with
    let result: ResultAndState<HaltReason> = ResultAndState {
        result: ExecutionResult::Success {
            reason: SuccessReason::Stop,
            gas_used: 500,
            gas_refunded: 0,
            logs: vec![],
            output: Output::Call(Bytes::from(vec![1, 2, 3])),
        },
        state: EvmState::default(),
    };

    // ORIGINAL ASSERTIONS
    // This is what would exist in the original test
    assert!(result.result.is_success());
    
    // 2. MIGRATION APPROACH
    // When migrating, we can use the migrate_test_to_testdata function
    // which will run the original assertions and also save the testdata
    migrate_test_to_testdata("migration_example", &result, |res| {
        // Include the original assertions here
        assert!(res.result.is_success());
    }).unwrap();

    // 3. FINAL MIGRATED TEST
    // After migration is complete, the test can be simplified to just:
    compare_or_save_testdata("migrated/migration_example.json", &result).unwrap();
}

/// Example of a test that has been fully migrated to use testdata comparison.
/// This is what tests should look like after migration is complete.
#[test]
fn fully_migrated_test() {
    // Create a minimal result and state
    let result: ResultAndState<HaltReason> = ResultAndState {
        result: ExecutionResult::Success {
            reason: SuccessReason::Stop,
            gas_used: 1000,
            gas_refunded: 0,
            logs: vec![],
            output: Output::Call(Bytes::from(vec![4, 5, 6])),
        },
        state: EvmState::default(),
    };

    // Simply use the testdata comparison utility
    // No assertions needed - full validation is done by comparing with testdata
    compare_or_save_testdata("fully_migrated_test.json", &result).unwrap();
} 
